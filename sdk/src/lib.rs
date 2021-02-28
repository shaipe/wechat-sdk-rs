//! copyright © shaipe 2021 - present
//! 微信系列对接处理公用工具类
//! created by shaipe 20210228

/// 字义微信结果类型
pub type WeChatResult<T> = Result<T, WeChatError>;

#[macro_use]
pub mod macros;

mod errors;
pub use errors::WeChatError;

mod wxcrypto;
pub use wxcrypto::{aes128_cbc_decrypt, aes256_cbc_decrypt, WeChatCrypto};

mod client;
pub use client::Client;

pub mod xmlutil;
// pub use xmlutil::

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}