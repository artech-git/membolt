

mod file; 
mod sample; 


#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::file::DirectIO;

    // use self::sample::DirectIO;

    use super::*;

    // #[test]
    // fn open_file() { 
    //     const N: usize = 577;
    //     let mut dio: DirectIO<N> = DirectIO::open("sample.txt").unwrap();
    //     let s = "kjfadjfsajfjsfjsadfjsdafjsdjfsdf".as_bytes();
    //     let y = dio.write(s).unwrap();
    //     dio.flush();
    //     assert_eq!(s.len(),y);
    // }

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
