use std::process::Termination;

// Maybe all this can be skipped and just implemented for the connector error

pub enum ExitReason {
    Success,
    Reconfiguration,
    Failure,
    Unknown,
}

impl From<ExitReason> for i32 {
    fn from(value: ExitReason) -> Self {
        match value {
            ExitReason::Success => 0,
            ExitReason::Failure => -1,
            ExitReason::Reconfiguration => -2,
            ExitReason::Unknown => -3,
        }
    }
}

impl Termination for ExitReason {
    fn report(self) -> std::process::ExitCode {
        match self {
            ExitReason::Success => std::process::ExitCode::SUCCESS,
            _ => std::process::ExitCode::FAILURE,
        }
    }
}
