//! Backend-only HTTP-0 native microkernel compute service.
//! Independent integer stencil execution, Fischer room addressing and
//! persistent result recall. No browser, external drive, network provider,
//! learned HRM checkpoint, JavaScript, unsafe code or floating arithmetic.
#![forbid(unsafe_code)]
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use crate::freeze::fischer_address;

const PRIME: u64 = 1_000_081;
const GENERATOR: u64 = 7;
pub const CELLS: usize = 27;
const TICKS: usize = 243;
const WIRE_SIZE: usize = 1 + 1 + 8 + CELLS;
const OUTPUT_SIZE: usize = CELLS * 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Task {
    pub id: u8,
    pub seed: u64,
    pub trits: [u8; CELLS],
}
impl Task {
    pub fn encode(&self)->Vec<u8>{
        let mut out=vec![1,self.id];
        out.extend_from_slice(&self.seed.to_le_bytes());
        out.extend_from_slice(&self.trits);
        out
    }
    fn parse(data:&[u8])->Result<Self,String>{
        if data.len()!=WIRE_SIZE || data[0]!=1 || data[1]>=54 {
            return Err("kernel task framing invalid".into());
        }
        let seed=u64::from_le_bytes(data[2..10].try_into()
            .map_err(|_|"invalid seed width")?);
        let trits:[u8;CELLS]=data[10..].try_into()
            .map_err(|_|"invalid trit count")?;
        if trits.iter().any(|x|*x>2){return Err("nonternary kernel coefficient".into());}
        Ok(Self{id:data[1],seed,trits})
    }
}
/// Historical HRM slow/fast shape classes; deterministic source-derived
/// planner. This is a route proxy, NOT a trained or loaded HRM model.
fn proxy_route(task:&Task, arity:usize)->usize{
    let mut zero=0_usize;
    let mut two=0_usize;
    let mut total=0_usize;
    for value in &task.trits{
        if *value==0 {zero+=1;}
        if *value==2 {two+=1;}
        total+=usize::from(*value);
    }
    let shape=(zero+3*two+total)%8;
    (usize::from(task.id)*7+13*shape+total)%arity
}
fn address_for(lane:usize)->u64{
    // Intentionally independent of the producer's square-and-multiply.
    let mut address=1;
    for _ in 0..lane {address=(address*GENERATOR)%PRIME;}
    address
}
fn compute_independent(task:&Task)->[u64;CELLS]{
    let mut current=[0_u64;CELLS];
    for (index,item) in current.iter_mut().enumerate(){
        let x=index as u64+1;
        *item=(task.seed%PRIME+13*x*x+17*u64::from(task.trits[index]))%PRIME;
    }
    for tick in 0..TICKS{
        let mut next=[0_u64;CELLS];
        // The reference calculator uses explicit neighbors; this one is a
        // three-term ring convolution with separately ordered operations.
        for index in 0..CELLS{
            let terms=[
                (7_u64,current[(index+CELLS-1)%CELLS]),
                (3_u64,current[index]),
                (5_u64,current[(index+1)%CELLS]),
            ];
            let mut value=11*(u64::from(task.trits[index])+(tick%3)as u64);
            for (weight,source) in terms {value+=weight*source;}
            next[index]=value%PRIME;
        }
        current=next;
    }
    current
}
#[derive(Default)]
struct Service{
    calls:usize,
    hits:usize,
    state:BTreeMap<(usize,u64),u64>,
    memo:BTreeMap<(usize,u64,u8,u64,[u8;CELLS]),[u64;CELLS]>,
}
impl Service{
    fn calculate(&mut self,route:&str,body:&[u8])->Result<Vec<u8>,String>{
        let parts:Vec<&str>=route.split('/').collect();
        if parts.len()!=6 || !parts[0].is_empty() || parts[1]!="compute"{
            return Err("native room route shape incorrect".into());
        }
        let mode:u8=parts[2].parse().map_err(|_|"invalid arm code")?;
        let arity:usize=parts[3].parse().map_err(|_|"invalid arity")?;
        let lane:usize=parts[4].parse().map_err(|_|"invalid lane")?;
        let given:u64=parts[5].parse().map_err(|_|"invalid Fischer address")?;
        if !matches!(mode,1..=3) || !matches!(arity,3|27|81) || lane>=arity{
            return Err("invalid room execution mode or arity".into());
        }
        let task=Task::parse(body)?;
        let right_lane=if mode==3 {proxy_route(&task,arity)}
            else {usize::from(task.id)%arity};
        if lane!=right_lane || given!=address_for(lane){
            return Err("independent Fischer / HRM room route mismatched".into());
        }
        let key=(arity,given,task.id,task.seed,task.trits);
        let cache=mode>=2;
        let found=if cache {self.memo.get(&key).copied()} else {None};
        let hit=found.is_some();
        let result=found.unwrap_or_else(||compute_independent(&task));
        if cache && !hit {self.memo.insert(key,result);}
        self.calls+=1;
        if hit {self.hits+=1;}
        let state=self.state.entry((arity,given)).or_insert(0);
        let mut fold=(*state)^given;
        for value in &result {
            fold=fold.rotate_left(7).wrapping_mul(0x100_0000_01b3)
                .wrapping_add(*value);
        }
        *state=fold;
        let mut out=Vec::with_capacity(1+OUTPUT_SIZE);
        out.push(u8::from(hit));
        for value in result {out.extend_from_slice(&value.to_le_bytes());}
        Ok(out)
    }
}
fn receive(socket:&mut TcpStream)->Result<(String,Vec<u8>),String>{
    let mut reader=BufReader::new(socket);
    let mut first=String::new();
    reader.read_line(&mut first).map_err(|e|e.to_string())?;
    if first.len()>180 || !first.ends_with("\r\n"){
        return Err("invalid HTTP-0 compute method".into());
    }
    let route=first.strip_prefix("POST ")
        .and_then(|s|s.strip_suffix(" HTTP/1.1\r\n"))
        .ok_or("noncanonical HTTP-0 compute request")?.to_owned();
    let mut content_length=None;
    let mut ended=false;
    for _ in 0..16{
        let mut line=String::new();
        reader.read_line(&mut line).map_err(|e|e.to_string())?;
        if line=="\r\n"{ended=true;break;}
        if line.len()>180 || !line.ends_with("\r\n"){
            return Err("malformed compute HTTP-0 header".into());
        }
        if let Some(n)=line.strip_prefix("Content-Length: "){
            if content_length.replace(n.trim().parse::<usize>()
                .map_err(|_|"content length value")?).is_some(){
                return Err("duplicate HTTP-0 content length".into());
            }
        }
    }
    if !ended{return Err("unterminated HTTP-0 headers".into());}
    let length=content_length.ok_or("missing content length")?;
    if length>WIRE_SIZE{return Err("compute request size exceeded".into());}
    let mut message=vec![0_u8;length];
    reader.read_exact(&mut message).map_err(|e|e.to_string())?;
    Ok((route,message))
}
fn reply(socket:&mut TcpStream,status:&str,data:&[u8])->Result<(),String>{
    let prefix=format!("HTTP/1.1 {status}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",data.len());
    socket.write_all(prefix.as_bytes()).map_err(|e|e.to_string())?;
    socket.write_all(data).map_err(|e|e.to_string())?;
    socket.flush().map_err(|e|e.to_string())
}
pub fn serve()->Result<(),String>{
    let listener=TcpListener::bind("127.0.0.1:0").map_err(|e|e.to_string())?;
    let port=listener.local_addr().map_err(|e|e.to_string())?.port();
    println!("PNP_HTTP0_COMPUTE_READY|port={port}|backend=ONLY|status=PASS|json=0");
    std::io::stdout().flush().map_err(|e|e.to_string())?;
    let mut service=Service::default();
    for stream in listener.incoming(){
        let mut socket=stream.map_err(|e|e.to_string())?;
        match receive(&mut socket){
            Ok((route,_)) if route=="/stop"=>{
                let info=format!("COMPUTE_STOP|calls={}|hits={}|rooms={}|entries={}|status=PASS|json=0\n",
                    service.calls,service.hits,service.state.len(),service.memo.len());
                reply(&mut socket,"200 OK",info.as_bytes())?;
                break;
            }
            Ok((route,body))=>{
                match service.calculate(&route,&body){
                    Ok(out)=>reply(&mut socket,"200 OK",&out)?,
                    Err(message)=>reply(&mut socket,"422 Unprocessable Content",message.as_bytes())?,
                }
            }
            Err(message)=>reply(&mut socket,"400 Bad Request",message.as_bytes())?,
        }
    }
    Ok(())
}
pub struct Client{process:Child,port:u16}
pub struct ResultFrame{pub values:[u64;CELLS],pub hit:bool}
pub struct Stop{pub calls:usize,pub hits:usize,pub rooms:usize,pub entries:usize}
impl Client{
    pub fn start()->Result<Self,String>{
        let mut process=Command::new(std::env::current_exe()
            .map_err(|e|e.to_string())?)
            .arg("compute-fabric-serve").stdout(Stdio::piped()).spawn()
            .map_err(|e|e.to_string())?;
        let output=process.stdout.take().ok_or("child pipe absent")?;
        let mut line=String::new();
        BufReader::new(output).read_line(&mut line).map_err(|e|e.to_string())?;
        let port=line.strip_prefix("PNP_HTTP0_COMPUTE_READY|port=")
            .and_then(|s|s.split('|').next())
            .ok_or("compute process not ready")?
            .parse::<u16>().map_err(|_|"compute port invalid")?;
        Ok(Self{process,port})
    }
    fn call(&self,route:&str,body:&[u8])->Result<(String,Vec<u8>),String>{
        let mut socket=TcpStream::connect(("127.0.0.1",self.port))
            .map_err(|e|e.to_string())?;
        let request=format!("POST {route} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len());
        socket.write_all(request.as_bytes()).map_err(|e|e.to_string())?;
        socket.write_all(body).map_err(|e|e.to_string())?;
        let mut reader=BufReader::new(socket);
        let mut status=String::new();
        reader.read_line(&mut status).map_err(|e|e.to_string())?;
        let mut size=None;
        let mut terminated=false;
        for _ in 0..12{
            let mut line=String::new();
            reader.read_line(&mut line).map_err(|e|e.to_string())?;
            if line=="\r\n"{terminated=true;break;}
            if let Some(raw)=line.strip_prefix("Content-Length: "){
                size=Some(raw.trim().parse::<usize>().map_err(|_|"response size")?);
            }
        }
        if !terminated{return Err("compute response headers incomplete".into());}
        let n=size.ok_or("no response content length")?;
        if n>4096{return Err("compute response too large".into());}
        let mut data=vec![0_u8;n];
        reader.read_exact(&mut data).map_err(|e|e.to_string())?;
        let mut extra=Vec::new();
        reader.read_to_end(&mut extra).map_err(|e|e.to_string())?;
        if !extra.is_empty(){return Err("compute extra response body".into());}
        Ok((status,data))
    }
    pub fn execute(&self,mode:u8,arity:usize,task:&Task,lane:usize)
        ->Result<ResultFrame,String>
    {
        let address=fischer_address((lane%27) as u64,(lane/27) as u64)
            .ok_or("Fischer room address outside domain")?;
        let path=format!("/compute/{mode}/{arity}/{lane}/{address}");
        let (status,bytes)=self.call(&path,&task.encode())?;
        if status!="HTTP/1.1 200 OK\r\n"{
            return Err(format!("compute fabric rejected frame: {}",String::from_utf8_lossy(&bytes)));
        }
        if bytes.len()!=OUTPUT_SIZE+1 || bytes[0]>1{
            return Err("noncanonical compute frame".into());
        }
        let mut result=[0_u64;CELLS];
        for (i,value) in result.iter_mut().enumerate(){
            *value=u64::from_le_bytes(bytes[1+8*i..1+8*i+8].try_into()
                .map_err(|_|"compute output word")?);
            if *value>=PRIME{return Err("out-of-domain stencil field element".into());}
        }
        Ok(ResultFrame{values:result,hit:bytes[0]==1})
    }
    pub fn mutated_route_rejected(&self,task:&Task,arity:usize)->Result<bool,String>{
        let lane=usize::from(task.id)%arity;
        let false_address=(fischer_address((lane%27) as u64,(lane/27) as u64)
            .ok_or("invalid route for negative")?+1)%PRIME;
        let (status,_)=self.call(&format!("/compute/1/{arity}/{lane}/{false_address}"),&task.encode())?;
        Ok(status=="HTTP/1.1 422 Unprocessable Content\r\n")
    }
    pub fn mutated_trit_rejected(&self,task:&Task,arity:usize)->Result<bool,String>{
        let lane=usize::from(task.id)%arity;
        let address=fischer_address((lane%27) as u64,(lane/27) as u64)
            .ok_or("negative Fischer address")?;
        let mut bytes=task.encode();
        bytes[10]=3;
        let (status,_)=self.call(&format!("/compute/1/{arity}/{lane}/{address}"),&bytes)?;
        Ok(status=="HTTP/1.1 422 Unprocessable Content\r\n")
    }
    pub fn stop(mut self)->Result<Stop,String>{
        let (status,payload)=self.call("/stop",&[])?;
        if status!="HTTP/1.1 200 OK\r\n"{return Err("compute stop rejected".into());}
        let text=String::from_utf8(payload).map_err(|_|"non-UTF8 stop receipt")?;
        let get=|key:&str|->Result<usize,String>{
            text.split('|').find_map(|part|part.strip_prefix(key))
                .ok_or_else(||format!("missing stop field {key}"))?
                .parse::<usize>().map_err(|_|"invalid stop integer".into())
        };
        let stats=Stop{
            calls:get("calls=")?,
            hits:get("hits=")?,
            rooms:get("rooms=")?,
            entries:get("entries=")?,
        };
        if !self.process.wait().map_err(|e|e.to_string())?.success(){
            return Err("compute worker failed process exit".into());
        }
        Ok(stats)
    }
}
impl Drop for Client{
    fn drop(&mut self){let _=self.process.kill();let _=self.process.wait();}
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn canonical_source_ternary_and_route_controls(){
        let t=Task{id:17,seed:789_101,trits:[1_u8;CELLS]};
        assert_eq!(Task::parse(&t.encode()).unwrap(),t);
        let mut bad=t.encode();bad[10]=3;
        assert!(Task::parse(&bad).is_err());
        for width in [3,27,81]{
            for lane in 0..width{
                assert_eq!(address_for(lane),fischer_address((lane%27)as u64,(lane/27)as u64).unwrap());
            }
            assert!(proxy_route(&t,width)<width);
        }
        let result=compute_independent(&t);
        assert!(result.iter().all(|v|*v<PRIME));
        let mut service=Service::default();
        let lane=usize::from(t.id)%27;
        let path=format!("/compute/2/27/{lane}/{}",address_for(lane));
        let once=service.calculate(&path,&t.encode()).unwrap();
        let twice=service.calculate(&path,&t.encode()).unwrap();
        assert_eq!(once[0],0);
        assert_eq!(twice[0],1);
        assert_eq!(&once[1..],&twice[1..]);
        assert_eq!(service.calls,2);
        assert_eq!(service.hits,1);
        assert_eq!(service.memo.len(),1);
    }
}
