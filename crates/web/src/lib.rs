use prost::Message;
use protocol::protocol::NetworkMessage;

pub fn add(left: usize, right: usize) -> usize {
    if let Err(e) = NetworkMessage::decode(vec![1, 2, 3].as_slice()) {
        println!("convert error {:?}", e);
    }
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
