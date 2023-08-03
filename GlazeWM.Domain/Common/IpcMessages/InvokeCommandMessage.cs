using CommandLine;

namespace GlazeWM.Domain.Common.IpcMessages
{
  [Verb(
    "command",
    HelpText = "Invoke a WM command (eg. `command \"focus workspace 1\"`)."
  )]
  public class InvokeCommandMessage
  {
    [Value(
      0,
      Required = true,
      HelpText = "WM command to run (eg. \"focus workspace 1\")"
    )]
    public string Command { get; set; }

    [Option(
      'c',
      "context-container-id",
      Required = false,
      HelpText = "ID of container to use as context."
    )]
    public string ContextContainerId { get; set; }
  }
}
