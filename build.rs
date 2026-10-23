// Migrationen sind ins Programm eingebettet (sqlx::migrate!) – bei Änderungen neu übersetzen.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
