use argon2rs::argon2i_simple;

pub fn hash_argon2<'a>(password: &'a str, salt: &'a str) -> String {
        argon2i_simple(&password, &salt).iter()
                               .map(|b| format!("{:02X}", b))
                               .collect()
}