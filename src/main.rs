fn main() {
    #[cfg(target_os = "android")]
    ashmem::test_in_rust();
}
