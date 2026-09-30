use argon2::{
  Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
  password_hash::{SaltString, rand_core::OsRng},
};

pub fn hash(plain: &str) -> anyhow::Result<String> {
  Ok(
    Argon2::default()
      .hash_password(plain.as_bytes(), &SaltString::generate(&mut OsRng))?
      .to_string(),
  )
}

pub fn verify(plain: &str, hash: &str) -> bool {
  // 注意：解析用 PasswordHash::new（结构体构造），不是 PasswordHasher（trait）
  PasswordHash::new(hash)
    .map(|h| {
      Argon2::default()
        .verify_password(plain.as_bytes(), &h)
        .is_ok()
    })
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hash_and_verify_roundtrip() {
    let h = hash("hunter2").unwrap();
    assert!(verify("hunter2", &h));
    assert!(!verify("wrong", &h));
  }
}
