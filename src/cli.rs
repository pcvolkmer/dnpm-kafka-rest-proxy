use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
#[command(arg_required_else_help(true))]
pub struct Cli {
    #[arg(
        long,
        alias = "kafka-servers",
        env = "APP_KAFKA_SERVERS",
        default_value = "kafka:9094",
        help = "Kafka Bootstrap Server"
    )]
    pub bootstrap_server: String,
    #[arg(
        long,
        alias = "kafka-topic",
        env = "APP_KAFKA_TOPIC",
        default_value = "etl-processor_input",
        help = "Kafka Topic"
    )]
    pub topic: String,
    #[arg(
        long,
        alias = "security-token",
        env = "APP_SECURITY_TOKEN",
        help = "bcrypt hashed Security Token"
    )]
    pub token: String,
    #[arg(
        long,
        env = "APP_LISTEN",
        default_value = "[::]:3000",
        help = "Address and port for HTTP requests"
    )]
    pub listen: String,
}
