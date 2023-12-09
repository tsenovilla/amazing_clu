use amazing_clu::Clu;

fn main(){
    match Clu::run(){
        Ok(output) => println!("{output}"),
        Err(error) => println!("{error}")
    }
}