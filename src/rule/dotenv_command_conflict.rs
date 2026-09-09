use super::*;

define_rule! {
  DotenvCommandConflictRule {
    id: "dotenv-command-conflict",
    message: "conflicting dotenv command setting",
    run(context) {
      context.conflicting_settings(|previous, current| {
        previous.name.value == "dotenv-command" && current.loads_dotenv()
          || current.name.value == "dotenv-command" && previous.loads_dotenv()
      })
    }
  }
}
