use crate::errors::WeChatError;
use crate::WeChatResult;
// use base64::*;
use byteorder::{NativeEndian, ReadBytesExt};
use crypto::buffer::{BufferResult, ReadBuffer, WriteBuffer};
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use crypto::{aes, blockmodes, buffer, symmetriccipher};
use std::collections::HashMap;
use std::io::Cursor;

#[derive(Debug, Eq, PartialEq)]
pub struct WeChatCrypto {
    token: String,
    key: Vec<u8>,
    _id: String,
}

impl WeChatCrypto {
    /// new
    pub fn new(token: &str, encoding_aes_key: &str, _id: &str) -> WeChatCrypto {
        let mut aes_key = encoding_aes_key.to_owned();
        aes_key.push('=');
        let key = base64::decode(&aes_key).unwrap();
        // println!("{:?}",encoding_aes_key);
        // let c= Config::new(CharacterSet::Crypt,true);
        // c.decode_allow_trailing_bits(true);

        // let key = base64::decode_config(&aes_key,c).unwrap();
        // println!("{:?}",key);
        WeChatCrypto {
            token: token.to_owned(),
            key: key,
            _id: _id.to_owned(),
        }
    }

    /// 获取签名
    fn get_signature(&self, timestamp: i64, nonce: &str, encrypted: &str) -> String {
        let mut data = vec![
            self.token.clone(),
            timestamp.to_string(),
            nonce.to_owned(),
            encrypted.to_owned(),
        ];
        data.sort();
        let data_str = data.join("");

        // sha1
        let mut hasher = Sha1::new();

        // write input message
        hasher.input_str(&data_str);

        // read hash digest
        hasher.result_str()
    }

    /// 消息解密
    pub fn decrypt_message(
        &self,
        xml: &str,
        query_params: HashMap<String, String>,
    ) -> WeChatResult<String> {
        //随机数
        let nonce = get_hash_value(&query_params, "nonce");
        //时间缀
        let timestamp = match get_hash_value(&query_params, "timestamp").parse::<i64>() {
            Ok(v) => v,
            Err(_e) => 0,
        };
        //签名信息
        let signature = get_hash_value(&query_params, "msg_signature");

        use super::xmlutil;
        let package = xmlutil::parse(xml);
        let doc = package.as_document();
        let encrypted_msg = xmlutil::evaluate(&doc, "//xml/Encrypt/text()").string();
        // println!("encrypted_msg={:?}",encrypted_msg);
        let real_signature = self.get_signature(timestamp, &nonce, &encrypted_msg);

        // println!("o: {}, new: {}", signature, real_signature);

        if signature != real_signature {
            return Err(WeChatError::InvalidSignature);
        }
        let msg = self.decrypt(&encrypted_msg)?;
        Ok(msg)
    }

    /// 解密
    pub fn decrypt(&self, ciphertext: &str) -> WeChatResult<String> {
        let b64decoded = base64::decode(ciphertext).unwrap();
        // aes descrypt
        let text = aes256_cbc_decrypt(&b64decoded, &self.key, &self.key[..16]).unwrap();

        let mut rdr = Cursor::new(text[16..20].to_vec());
        let content_length = u32::from_be(rdr.read_u32::<NativeEndian>().unwrap()) as usize;
        let content = &text[20..content_length + 20];
        let from_id = &text[content_length + 20..];
        // println!("form_id: {:?}  ,, id_ {:?}", from_id, self._id.as_bytes());
        // 此处取出的formid中包含了回车符,只能取前18位进行判断比较
        if &from_id[0..18] != self._id.as_bytes() {
            return Err(WeChatError::InvalidAppId);
        }
        let content_string = String::from_utf8(content.to_vec()).unwrap();
        Ok(content_string)
    }

    // pub fn encrypt_message(&self, msg: &str, timestamp: i64, nonce: &str) -> WeChatResult<String> {
    //     let prp = PrpCrypto::new(&self.key);
    //     let encrypted_msg = try!(prp.encrypt(msg, &self._id));
    //     let signature = self.get_signature(timestamp, nonce, &encrypted_msg);
    //     let msg = format!(
    //         "<xml>\n\
    //         <Encrypt><![CDATA[{encrypt}]]></Encrypt>\n\
    //         <MsgSignature><![CDATA[{signature}]]></MsgSignature>\n\
    //         <TimeStamp>{timestamp}</TimeStamp>\n\
    //         <Nonce><![CDATA[{nonce}]]></Nonce>\n\
    //         </xml>",
    //         encrypt=encrypted_msg,
    //         signature=signature,
    //         timestamp=timestamp,
    //         nonce=nonce,
    //     );
    //     Ok(msg)
    // }
}

/// 从HashMap中取值
fn get_hash_value(query_params: &HashMap<String, String>, key: &str) -> String {
    match query_params.get(key) {
        Some(val) => val.clone(),
        None => "".to_owned(),
    }
}

/// Decrypts a buffer with the given key and iv using AES-256/CBC/Pkcs encryption.
pub fn aes256_cbc_decrypt(
    encrypted_data: &[u8],
    key: &[u8],
    iv: &[u8],
) -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {
    // 此处的最后一个参数要使用不直充的方式才行
    let mut decryptor =
        aes::cbc_decryptor(aes::KeySize::KeySize256, key, iv, blockmodes::NoPadding);
    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(encrypted_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);
    loop {
        match decryptor.decrypt(&mut read_buffer, &mut write_buffer, true) {
            Ok(result) => {
                final_result.extend(
                    write_buffer
                        .take_read_buffer()
                        .take_remaining()
                        .iter()
                        .map(|&i| i),
                );
                match result {
                    BufferResult::BufferUnderflow => break,
                    BufferResult::BufferOverflow => {}
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
    Ok(final_result)
}

#[cfg(test)]
mod tests {
    use super::WeChatCrypto;

    #[test]
    fn test_decrypt_message() {
        let xml = "<xml>
        <AppId><![CDATA[wxce775970ff046a47]]></AppId>
        <Encrypt><![CDATA[hcks7uOOFfE5iw2PAYnf5uSboGVxG4YPL+1M+eKcW53YFSpwXk4T/0qiLTQjeIzNxfC6kTFEo0Ti8vVF4D+3A/ORcDLL6SAeZu1Cvzh0NQ0JZBfP5scSMYFl97EqDDNfR9aQjoA/3zM4wnGq0FPcpNjrtcNcVRUMtMOiMBaWyb3Lh1ZbtSMtvRPDJgjiGjB3BOeqlImQj9nBz5XiVeHiXFGbyo57WtWQO4s+UTWh0UR3uWh4dPIwgKax/39nrbmOH6I87vfZyi2JZLai8TyWupm76rkg/ATOcdegyREzbYzJLJOvc5C53H6Vre0sxvM8FM2+vL0+DrLew8taPi2iRA==]]>
        </Encrypt></xml>";
        let crypto = WeChatCrypto::new("shaipe", "kdjCGGJKSRjjhESfPO5lTSWtYS0v5pQX47skCkZczip", "wxce775970ff046a47");
        use std::collections::HashMap;
        let mut dic=HashMap::new();
        dic.insert("msg_signature".to_owned(),"6be15b5bf237498acd65b70d4532aa7cdcfdeab7".to_owned());
        dic.insert("nonce".to_owned(),"1525763395".to_owned());
        dic.insert("timestamp".to_owned(),"1586832708".to_owned());
        let decrypted = crypto
            .decrypt_message(xml, dic)
            .unwrap();
        println!("decrypted={:?}", decrypted);
    }
}
