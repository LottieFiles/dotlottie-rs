#[cfg(feature = "encryption")]
mod test_utils;

#[cfg(feature = "encryption")]
mod tests {
    use crate::test_utils::{HEIGHT, WIDTH};
    use dotlottie_rs::{ColorSpace, Player, PlayerError};
    use std::io;

    const PASSWORD: &str = "s3cr3t-pa55";

    fn encrypt(plaintext: &[u8], password: &str) -> Vec<u8> {
        use zip::write::SimpleFileOptions;
        use zip::{AesMode, CompressionMethod, ZipArchive, ZipWriter};

        let mut src = ZipArchive::new(io::Cursor::new(plaintext.to_vec())).unwrap();
        let mut out = ZipWriter::new(io::Cursor::new(Vec::new()));
        for i in 0..src.len() {
            let mut e = src.by_index(i).unwrap();
            let name = e.name().to_string();
            let opts = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .with_aes_encryption(AesMode::Aes256, password);
            out.start_file(name, opts).unwrap();
            io::copy(&mut e, &mut out).unwrap();
        }
        out.finish().unwrap().into_inner()
    }

    fn encrypted_fixture() -> Vec<u8> {
        let plaintext = std::fs::read("assets/animations/dotlottie/v2/image.lottie").unwrap();
        encrypt(&plaintext, PASSWORD)
    }

    #[test]
    fn loads_with_correct_password() {
        let enc = encrypted_fixture();
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
            .unwrap();
        assert!(player
            .load_dotlottie_data_with_password(&enc, PASSWORD)
            .is_ok());
        assert!(
            player.is_loaded(),
            "player should be loaded after decryption"
        );
    }

    #[test]
    fn wrong_password_reports_invalid_password() {
        let enc = encrypted_fixture();
        let mut player = Player::new();
        assert_eq!(
            player.load_dotlottie_data_with_password(&enc, "nope"),
            Err(PlayerError::InvalidPassword)
        );
    }

    #[test]
    fn no_password_reports_encrypted_archive() {
        let enc = encrypted_fixture();
        let mut player = Player::new();
        assert_eq!(
            player.load_dotlottie_data(&enc),
            Err(PlayerError::EncryptedArchive)
        );
    }
}
