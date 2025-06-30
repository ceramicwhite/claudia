// Example usage of type-safe agent module features

use crate::commands::agents::{AgentId, RunId, SessionId, AgentCreate, ModelType};

#[cfg(test)]
mod examples {
    use super::*;

    #[test]
    fn example_newtype_wrappers() {
        // Creating type-safe IDs
        let agent_id = AgentId::new(1).expect("Valid agent ID");
        let run_id = RunId::new(42).expect("Valid run ID");
        let session_id = SessionId::generate(); // Auto-generates UUID
        
        // Accessing inner values
        println!("Agent ID: {}", agent_id.inner());
        println!("Run ID: {}", run_id.inner());
        println!("Session ID: {}", session_id.inner());
        
        // Display implementation
        println!("Agent: {}", agent_id);
        
        // Type safety prevents mixing IDs
        // let wrong: AgentId = run_id; // Compile error!
    }
    
    #[test]
    fn example_builder_pattern() {
        // Creating an agent with the builder
        let agent = AgentCreate::builder()
            .name("Code Assistant")
            .icon("💻")
            .system_prompt("You are a helpful coding assistant")
            .default_task("Help me write better code")
            .model(ModelType::Opus4)
            .sandbox_enabled(true)
            .enable_file_read(true)
            .enable_file_write(true)
            .enable_network(false)
            .build()
            .expect("Valid agent configuration");
        
        // Validation happens at build time
        agent.validate().expect("Agent is valid");
    }
    
    #[test]
    fn example_builder_validation() {
        // Missing required fields
        let result = AgentCreate::builder()
            .name("Test Agent")
            // Missing icon and system_prompt
            .build();
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Agent icon is required");
        
        // Empty name validation
        let result = AgentCreate::builder()
            .name("   ") // Whitespace only
            .icon("🤖")
            .system_prompt("Test prompt")
            .build();
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty or whitespace"));
    }
    
    #[test]
    fn example_session_id_validation() {
        // Valid UUID
        let valid = SessionId::new("550e8400-e29b-41d4-a716-446655440000".to_string());
        assert!(valid.is_ok());
        
        // Invalid format
        let invalid = SessionId::new("not-a-uuid".to_string());
        assert!(invalid.is_err());
        assert!(invalid.unwrap_err().contains("Invalid session ID format"));
        
        // Empty string
        let empty = SessionId::new("".to_string());
        assert!(empty.is_err());
        assert_eq!(empty.unwrap_err(), "Session ID cannot be empty");
    }
    
    #[test]
    fn example_id_parsing() {
        use std::str::FromStr;
        
        // Parse from string
        let agent_id = AgentId::from_str("123").expect("Valid ID");
        assert_eq!(agent_id.inner(), 123);
        
        // Invalid format
        let invalid = AgentId::from_str("abc");
        assert!(invalid.is_err());
        
        // Negative ID
        let negative = AgentId::from_str("-1");
        assert!(negative.is_err());
        assert_eq!(negative.unwrap_err(), "Agent ID must be positive");
    }
}

// Example function showing type safety in practice
pub fn process_agent_run(agent_id: AgentId, run_id: RunId, session_id: SessionId) {
    // Type safety ensures correct parameter passing
    println!("Processing run {} for agent {} in session {}", 
             run_id, agent_id, session_id);
    
    // Can't accidentally swap parameters due to different types
    // process_agent_run(run_id, agent_id, session_id); // Compile error!
}

// Example of using the builder in actual code
pub fn create_default_assistant() -> Result<AgentCreate, String> {
    AgentCreate::builder()
        .name("General Assistant")
        .icon("🤖")
        .system_prompt("You are a helpful AI assistant. Be concise and accurate.")
        .model(ModelType::Sonnet4)
        .sandbox_enabled(true)
        .enable_file_read(true)
        .enable_file_write(false)
        .enable_network(false)
        .build()
}