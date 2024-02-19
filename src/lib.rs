

mod file; 


#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::file::DirectIO;

    use super::*;

    #[test]
    fn develop_file() {

        const N: usize = 512; 
        const A: usize = !(N - 1);

        let mut dio: DirectIO<N, A> = DirectIO::open("./asss.txt").unwrap(); 
        let data = [4; 623];
        let y = dio.write(&data).unwrap();
        dio.flush(); 
        assert_eq!(data.len(), y);  
    }
}


use file::*; 