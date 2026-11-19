//! Schema-Migrationen, ins Programm eingebettet. Jede Fassung liegt als eigenes Modul vor;
//! sqlx merkt sich in `_sqlx_migrations`, was schon gelaufen ist, und prüft die Prüfsumme –
//! einmal ausgelieferte Migrationen darum nie mehr ändern, sondern eine neue anhängen.

use sqlx::migrate::{Migration, MigrationType, Migrator};
use sqlx::SqlStr;

mod m0001_init;
mod m0002_scan;
mod m0003_mail_folders_thumbnails;
mod m0004_reminders_folders_printers;

/// (Version, Beschreibung, SQL) – in der Reihenfolge, in der sie laufen.
const ALL: &[(i64, &str, &str)] = &[
    (1, "init", m0001_init::SRC),
    (2, "scan", m0002_scan::SRC),
    (3, "mail folders thumbnails", m0003_mail_folders_thumbnails::SRC),
    (4, "reminders folders printers", m0004_reminders_folders_printers::SRC),
];

pub fn migrator() -> Migrator {
    Migrator::with_migrations(
        ALL.iter()
            .map(|&(version, description, sql)| {
                Migration::new(
                    version,
                    description.into(),
                    MigrationType::Simple,
                    SqlStr::from_static(sql),
                    sql.starts_with("-- no-transaction"),
                )
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    /// Prüfsummen der ausgelieferten Fassungen – ändert sich eine, verweigern bestehende
    /// Datenbanken den Start.
    const SHA384: &[&str] = &[
        "36f28f5e388fd37a6bbd00a394e09a09b7d11bb9c47ebc94e61ac0635328e5cb9837f82bf1e510b027798f0c2eaaaed0",
        "39e05198be40f71a9b92fbe18b29c35f00314f32a4c3711dddaf66b347620dc35d92ebc63d3ea406e55583004a78c3d5",
        "4def1b9ebdc1588a67896732850c01772bd7db70b3948e227183a33fa1830d53b184173c30a47ac48bcd6207a8b17667",
        "a01aab0039e5800c8513ef909a3dc48d80602822a3b7bcdb748145c4abd777d929c5be277c6202588a99288cdbe2733d",
    ];

    #[test]
    fn released_migrations_unchanged() {
        let m = super::migrator();
        assert!(m.migrations.len() >= SHA384.len());
        for (i, (mig, sum)) in m.migrations.iter().zip(SHA384).enumerate() {
            assert_eq!(mig.version, i as i64 + 1);
            assert_eq!(hex::encode(&mig.checksum), *sum, "Migration {}", mig.version);
        }
    }
}
