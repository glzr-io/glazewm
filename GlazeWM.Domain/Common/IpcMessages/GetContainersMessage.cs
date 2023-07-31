using CommandLine;

namespace GlazeWM.Domain.Common.IpcMessages
{
  [Verb(
    "containers",
    HelpText = "Get all containers (monitors, workspaces, windows, split containers)."
  )]
  public class GetContainersMessage
  {
  }
}
