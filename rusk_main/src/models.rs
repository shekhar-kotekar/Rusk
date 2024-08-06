use std::str;

#[derive(Debug, PartialEq)]
pub enum ProcessorType {
    AddOne,
    RandomNumberGenerator,
    UpperCase,
}

impl From<&str> for ProcessorType {
    fn from(processor_type: &str) -> Self {
        match processor_type.to_lowercase().as_str() {
            "add_one" => ProcessorType::AddOne,
            "random_number_generator" => ProcessorType::RandomNumberGenerator,
            "uppercase" => ProcessorType::UpperCase,
            other => panic!("Invalid processor type: {}", other),
        }
    }
}

impl ToString for ProcessorType {
    fn to_string(&self) -> String {
        match self {
            ProcessorType::AddOne => String::from("add_one"),
            ProcessorType::RandomNumberGenerator => String::from("random_number_generator"),
            ProcessorType::UpperCase => String::from("uppercase"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ServerAction {
    CreateProcessor,
    StartProcessor,
    StopProcessor,
    DeleteProcessor,
    ConnectProcessors,
    DisconnectProcessors,
    DeleteConnection,
    GetProcessorStatus,
}

impl From<&str> for ServerAction {
    fn from(action: &str) -> Self {
        match action.to_lowercase().as_str() {
            "createprocessor" => ServerAction::CreateProcessor,
            "startprocessor" => ServerAction::StartProcessor,
            "stopprocessor" => ServerAction::StopProcessor,
            "deleteprocessor" => ServerAction::DeleteProcessor,
            "connectprocessors" => ServerAction::ConnectProcessors,
            "disconnectprocessors" => ServerAction::DisconnectProcessors,
            "deleteconnection" => ServerAction::DeleteConnection,
            "getprocessorstatus" => ServerAction::GetProcessorStatus,
            other => panic!("Invalid action type: {}", other),
        }
    }
}

pub struct ServerMessage {
    pub action: ServerAction,
    pub sub_commands: Vec<String>,
}

impl ServerMessage {
    pub fn parse(message_buffer: &[u8]) -> Self {
        let message = str::from_utf8(message_buffer).unwrap();
        let message_parts: Vec<&str> = message.split("\r\n").collect();
        assert!(
            message_parts.len() >= 2,
            "Invalid message. Expected message format: action\r\nfrom\r\n<to>"
        );
        let action = ServerAction::from(message_parts[0]);
        ServerMessage {
            action,
            sub_commands: message_parts[1..].iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[cfg(test)]
mod models_tests {
    use super::*;

    #[test]
    fn test_parse_message() {
        let message = "createProcessor\r\nR1";
        let message_buffer = message.as_bytes();
        let message = ServerMessage::parse(message_buffer);
        assert_eq!(message.action, ServerAction::CreateProcessor);
        assert_eq!(message.sub_commands, vec!["R1".to_string()]);
    }

    #[test]
    fn test_parse_message_should_parse_connection_messages() {
        let message = "connectProcessors\r\nR1\r\nR2";
        let message_buffer = message.as_bytes();
        let message = ServerMessage::parse(message_buffer);
        assert_eq!(message.action, ServerAction::ConnectProcessors);
        assert_eq!(
            message.sub_commands,
            vec!["R1".to_string(), "R2".to_string()]
        );
    }
}
