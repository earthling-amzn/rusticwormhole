mod test {
    use std::{fs};
    use std::fs::ReadDir;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex, MutexGuard, Condvar};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn list_files() {
        let p = PathBuf::from("/tmp");
        let mut v = vec![p];
        let mutex = Mutex::from(v);
        // walk_directory(&mutex);
    }

    #[test]
    fn threads_list_files() {
        let p = PathBuf::from("/tmp");
        let mut v = vec![p];
        let mut threads = vec![];
        let mutex = Arc::new((Mutex::new(v), Condvar::new()));
        for _ in 0..8 {
            let stack = mutex.clone();
            let thread = thread::spawn(move ||{
               walk_directory(&stack);
            });
            threads.push(thread);
        }

        for thread in threads {
            thread.join().unwrap();
        }
    }

    fn walk_directory(mutex: &(Mutex<Vec<PathBuf>>, Condvar)) {
        loop {
            match locked_pop(&mutex) {
                None => {
                    println!("{:?} Exiting", thread::current().id());
                    break;
                },
                Some(p) => {
                    if p.is_dir() {
                        handle_directory(&mutex, &p);
                    } else {
                        println!("{:?} Work on {:?}", thread::current().id(), p.as_path());
                    }
                }
            }
        }
    }

    fn locked_pop(arc: &(Mutex<Vec<PathBuf>>, Condvar)) -> Option<PathBuf> {
        let (mutex, condvar) = arc;
        let mut vector = mutex.lock().unwrap();
        if vector.is_empty() {
            let (mut vector, _) = condvar.wait_timeout(vector, Duration::from_millis(100)).unwrap();
            return if vector.is_empty() {
                None
            } else {
                vector.pop()
            }
        }
        vector.pop()
    }


    fn handle_directory(mutex: &(Mutex<Vec<PathBuf>>, Condvar), p: &PathBuf) {
        match fs::read_dir(p) {
            Err(_) => {},
            Ok(listing) => {
                enqueue_children(mutex, listing);
            }
        }
    }

    fn enqueue_children(pair: &(Mutex<Vec<PathBuf>>, Condvar), listing: ReadDir) {
        let (mutex, condvar) = pair;
        for entry in listing {
            match entry {
                Err(_) => {},
                Ok(f) => {
                    mutex.lock().unwrap().push(f.path());
                    condvar.notify_all();
                }
            }
        }
    }
}