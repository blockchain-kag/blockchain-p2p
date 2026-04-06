use std::{
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

use crate::node::types::node_command::NodeCommand;

pub struct NodeCommandHandler {
    input_receiver: Receiver<String>,
    event_sender: Sender<NodeCommand>,
    emmitter: Sender<String>,
}

impl NodeCommandHandler {
    pub fn new(
        input_receiver: Receiver<String>,
        event_sender: Sender<NodeCommand>,
        emmitter: Sender<String>,
    ) -> Self {
        Self {
            input_receiver,
            event_sender,
            emmitter,
        }
    }

    pub fn run(self) -> JoinHandle<()> {
        thread::spawn(move || {
            while let Ok(input) = self.input_receiver.recv() {
                match Self::parse_command(&input) {
                    Ok(cmd) => {
                        let _ = self.event_sender.send(cmd);
                    }
                    Err(err) => {
                        self.emmitter.send(format!("{}: {}", err, input)).unwrap();
                    }
                }
            }
        })
    }

    fn parse_command(input: &str) -> Result<NodeCommand, String> {
        let mut parsed = input.split_whitespace();

        match parsed.next() {
            Some("quit") => Ok(NodeCommand::Quit),
            Some("help") => Ok(NodeCommand::Help),
            Some("send") => {
                let to = parsed.next().unwrap().to_string();
                let amount = parsed.next().and_then(|v| v.parse::<u64>().ok()).unwrap();
                Ok(NodeCommand::Transfer(to, amount))
            }
            Some("start_mining") => {
                let minners = parsed
                    .next()
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(1);
                Ok(NodeCommand::StartMining(minners))
            }
            Some("stop_mining") => Ok(NodeCommand::StopMining),
            Some("pause_mining") => Ok(NodeCommand::PauseMining),
            Some("resume_mining") => Ok(NodeCommand::ResumeMining),
            Some("sync") => Ok(NodeCommand::StartSyncing),
            _ => Err("Unknown command".to_string()),
        }
    }
}
