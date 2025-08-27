use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
#[command(arg_required_else_help(true))]
pub struct Cli {
    #[arg(
        long,
        env = "LISTEN",
        default_value = "[::]:3000",
        help = "Address and port for HTTP requests"
    )]
    pub listen: String,
    #[arg(
        long,
        alias = "security-token",
        env = "SECURITY_TOKEN",
        help = "bcrypt hashed Security Token"
    )]
    pub token: String,
    #[arg(
        long,
        alias = "kafka-servers",
        env = "KAFKA_BOOTSTRAP_SERVERS",
        default_value = "kafka:9094",
        help = "Kafka Bootstrap Server"
    )]
    pub bootstrap_server: String,
    #[arg(
        long,
        alias = "kafka-topic",
        env = "KAFKA_TOPIC",
        default_value = "etl-processor_input",
        help = "Kafka Topic"
    )]
    pub topic: String,
    #[arg(
        long,
        env = "KAFKA_SSL_CA_FILE",
        help = "CA file for SSL connection to Kafka"
    )]
    pub ssl_ca_file: Option<String>,
    #[arg(
        long,
        env = "KAFKA_SSL_CERT_FILE",
        help = "Certificate file for SSL connection to Kafka"
    )]
    pub ssl_cert_file: Option<String>,
    #[arg(
        long,
        env = "KAFKA_SSL_KEY_FILE",
        help = "Key file for SSL connection to Kafka"
    )]
    pub ssl_key_file: Option<String>,
    #[arg(long, env = "KAFKA_SSL_KEY_PASSWORD", help = "The SSL key password")]
    pub ssl_key_password: Option<String>,
}
