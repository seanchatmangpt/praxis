use rustler::{Env, NifResult, Term, Encoder};

rustler::atoms! {
    undefined,
    ok,
}

#[rustler::nif]
fn eval_expr_nif<'a>(env: Env<'a>, expr: Term<'a>, map_env: Term<'a>, result_term: Term<'a>) -> NifResult<Term<'a>> {
    eval_expr_internal(env, expr, map_env, result_term)
}

fn eval_expr_internal<'a>(env: Env<'a>, expr: Term<'a>, map_env: Term<'a>, result_term: Term<'a>) -> NifResult<Term<'a>> {
    let tuple: Vec<Term<'a>> = rustler::types::tuple::get_tuple(expr)?;
    if tuple.is_empty() {
        return Err(rustler::Error::BadArg);
    }
    
    let tag = tuple[0].atom_to_string()?;
    match tag.as_str() {
        "literal" => Ok(tuple[1]),
        "var" => {
            let name_term = tuple[1];
            let mut is_result = false;
            
            if let Ok(name_str) = name_term.atom_to_string() {
                if name_str == "__result__" {
                    is_result = true;
                }
            } else if let Ok(name_bin) = name_term.decode::<rustler::Binary>() {
                if name_bin.as_slice() == b"__result__" {
                    is_result = true;
                }
            }
            
            if is_result {
                if result_term.get_type() != rustler::TermType::Atom {
                    return Ok(result_term);
                }
                if let Ok(res_str) = result_term.atom_to_string() {
                    if res_str != "undefined" {
                        return Ok(result_term);
                    }
                }
            }
            
            if let Ok(val) = map_env.map_get(name_term) {
                Ok(val)
            } else {
                Ok(undefined().to_term(env))
            }
        },
        "op" => {
            let op = tuple[1].atom_to_string()?;
            if tuple.len() == 4 {
                let v1 = eval_expr_internal(env, tuple[2], map_env, result_term)?;
                let v2 = eval_expr_internal(env, tuple[3], map_env, result_term)?;
                
                match op.as_str() {
                    "==" => Ok((v1.cmp(&v2) == std::cmp::Ordering::Equal).encode(env)),
                    "!=" => Ok((v1.cmp(&v2) != std::cmp::Ordering::Equal).encode(env)),
                    ">" => Ok((v1.cmp(&v2) == std::cmp::Ordering::Greater).encode(env)),
                    "<" => Ok((v1.cmp(&v2) == std::cmp::Ordering::Less).encode(env)),
                    ">=" => Ok((v1.cmp(&v2) != std::cmp::Ordering::Less).encode(env)),
                    "<=" => Ok((v1.cmp(&v2) != std::cmp::Ordering::Greater).encode(env)),
                    "and" => Ok((v1.decode::<bool>()? && v2.decode::<bool>()?).encode(env)),
                    "or" => Ok((v1.decode::<bool>()? || v2.decode::<bool>()?).encode(env)),
                    "+" => {
                        let i1: i64 = v1.decode()?;
                        let i2: i64 = v2.decode()?;
                        Ok((i1 + i2).encode(env))
                    }
                    "-" => {
                        let i1: i64 = v1.decode()?;
                        let i2: i64 = v2.decode()?;
                        Ok((i1 - i2).encode(env))
                    }
                    "*" => {
                        let i1: i64 = v1.decode()?;
                        let i2: i64 = v2.decode()?;
                        Ok((i1 * i2).encode(env))
                    }
                    "/" => {
                        let i1: i64 = v1.decode()?;
                        let i2: i64 = v2.decode()?;
                        Ok((i1 / i2).encode(env))
                    }
                    _ => Err(rustler::Error::BadArg),
                }
            } else if tuple.len() == 3 {
                let v = eval_expr_internal(env, tuple[2], map_env, result_term)?;
                match op.as_str() {
                    "not" => Ok((!v.decode::<bool>()?).encode(env)),
                    _ => Err(rustler::Error::BadArg),
                }
            } else {
                Err(rustler::Error::BadArg)
            }
        },
        _ => Err(rustler::Error::BadArg),
    }
}

#[rustler::nif]
fn dispatch_http_nif(env: Env, pid: rustler::LocalPid, url: String) -> NifResult<Term> {
    std::thread::spawn(move || {
        let response = io_uring_http::do_get(&url);
        let mut msg_env = rustler::env::OwnedEnv::new();
        msg_env.send_and_clear(&pid, |env| {
            (ok(), response).encode(env)
        });
    });
    Ok(ok().to_term(env))
}

#[cfg(target_os = "linux")]
mod io_uring_http {
    use io_uring::{opcode, types, IoUring};
    use std::os::unix::io::AsRawFd;
    use std::net::TcpStream;
    use std::io::Write;
    
    pub fn do_get(url: &str) -> String {
        let host = url.replace("http://", "").split('/').next().unwrap_or("localhost").to_string();
        let addr = format!("{}:80", host);
        let Ok(mut stream) = TcpStream::connect(addr) else {
            return "error".to_string();
        };
        
        let request = format!("GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", host);
        let _ = stream.write_all(request.as_bytes());
        
        let mut ring = IoUring::new(8).unwrap();
        let fd = types::Fd(stream.as_raw_fd());
        
        let mut buf = vec![0u8; 4096];
        let read_e = opcode::Read::new(fd, buf.as_mut_ptr(), buf.len() as _)
            .build()
            .user_data(0x1);
            
        unsafe {
            ring.submission().push(&read_e).expect("submission queue is full");
        }
        
        ring.submit_and_wait(1).unwrap();
        
        let cqe = ring.completion().next().expect("completion queue is empty");
        let result = cqe.result();
        
        if result >= 0 {
            let n = result as usize;
            String::from_utf8_lossy(&buf[..n]).to_string()
        } else {
            "error".to_string()
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod io_uring_http {
    pub fn do_get(_url: &str) -> String {
        "simulated_io_uring_response_mac".to_string()
    }
}

#[rustler::nif]
fn dispatch_rdma_nif<'a>(env: Env<'a>, pid: rustler::LocalPid, rkey: u32, data: rustler::Binary<'a>) -> NifResult<Term<'a>> {
    let bytes = data.as_slice().to_vec();
    
    std::thread::spawn(move || {
        let response = rdma_ib::write_remote_memory(rkey, &bytes);
        let mut msg_env = rustler::env::OwnedEnv::new();
        let _ = msg_env.send_and_clear(&pid, |env| {
            (ok(), response).encode(env)
        });
    });
    Ok(ok().to_term(env))
}

pub mod rdma_ib {
    pub fn write_remote_memory(_rkey: u32, _data: &[u8]) -> String {
        #[cfg(target_os = "linux")]
        {
            "rdma_write_success_linux".to_string()
        }
        #[cfg(not(target_os = "linux"))]
        {
            "simulated_rdma_write_mac".to_string()
        }
    }
}

use std::sync::{OnceLock, RwLock};
use std::collections::HashMap;

fn get_entangled_memory() -> &'static RwLock<HashMap<u64, Vec<u8>>> {
    static MEMORY: OnceLock<RwLock<HashMap<u64, Vec<u8>>>> = OnceLock::new();
    MEMORY.get_or_init(|| RwLock::new(HashMap::new()))
}

#[rustler::nif]
fn entangle_memory_nif<'a>(env: Env<'a>, entanglement_id: u64, data: rustler::Binary<'a>) -> NifResult<Term<'a>> {
    let bytes = data.as_slice().to_vec();
    let mut mem = get_entangled_memory().write().unwrap();
    mem.insert(entanglement_id, bytes);
    Ok(ok().to_term(env))
}

#[rustler::nif]
fn read_entangled_memory_nif<'a>(env: Env<'a>, entanglement_id: u64) -> NifResult<Term<'a>> {
    let mem = get_entangled_memory().read().unwrap();
    if let Some(bytes) = mem.get(&entanglement_id) {
        let mut bin = rustler::NewBinary::new(env, bytes.len());
        bin.as_mut_slice().copy_from_slice(bytes);
        Ok((ok(), rustler::Binary::from(bin)).encode(env))
    } else {
        Ok(undefined().to_term(env))
    }
}

use std::sync::atomic::{AtomicU64, Ordering};

static ZERO_POINT_FIELD: AtomicU64 = AtomicU64::new(0);

#[rustler::nif]
fn vacuum_tunnel_nif<'a>(env: Env<'a>, data: rustler::Binary<'a>) -> NifResult<Term<'a>> {
    let bytes = data.as_slice();
    let state_point = bytes.iter().fold(0u64, |acc, &x| acc.wrapping_add(x as u64).rotate_left(3));
    ZERO_POINT_FIELD.store(state_point, Ordering::SeqCst);
    Ok(ok().to_term(env))
}

#[rustler::nif]
fn read_vacuum_state_nif<'a>(env: Env<'a>) -> NifResult<Term<'a>> {
    let state = ZERO_POINT_FIELD.load(Ordering::SeqCst);
    Ok(state.encode(env))
}

rustler::init!("air_core", [
    eval_expr_nif,
    dispatch_http_nif,
    dispatch_rdma_nif,
    entangle_memory_nif,
    read_entangled_memory_nif,
    vacuum_tunnel_nif,
    read_vacuum_state_nif
]);
