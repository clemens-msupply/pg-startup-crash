use pq_sys::*;
use std::ffi::{CStr, CString};
use std::sync::{Arc, Barrier};
use std::thread;

fn last_error_message(conn: *const PGconn) -> String {
    unsafe {
        let error_ptr = PQerrorMessage(conn);
        let bytes = CStr::from_ptr(error_ptr).to_bytes();
        String::from_utf8_lossy(bytes).to_string()
    }
}

fn main() {
    const THREAD_COUNT: usize = 10;

    let db_url = "postgres://localhost/diesel_test";

    let b = Arc::new(Barrier::new(THREAD_COUNT));
    let mut handles = Vec::new();
    for i in 0..THREAD_COUNT {
        let b = Arc::clone(&b);

        let h = thread::spawn(move || {
            let connection_string = CString::new(db_url).unwrap();
            b.wait();
            let connection_ptr = unsafe { PQconnectdb(connection_string.as_ptr()) };
            let connection_status = unsafe { PQstatus(connection_ptr) };

            match connection_status {
                ConnStatusType::CONNECTION_OK => {
                    println!("Successfully initilized connection {}", i);

                    unsafe { PQfinish(connection_ptr) }
                }
                _ => {
                    let message = last_error_message(connection_ptr);
                    println!("Failed to initializeed connection {}: {}", i, message);

                    if !connection_ptr.is_null() {
                        // Note that even if the server connection attempt fails (as indicated by PQstatus),
                        // the application should call PQfinish to free the memory used by the PGconn object.
                        // https://www.postgresql.org/docs/current/libpq-connect.html
                        unsafe { PQfinish(connection_ptr) }
                    }
                }
            }
        });

        handles.push(h);
    }

    for h in handles {
        h.join().unwrap();
    }
}
