using CommandLine;

namespace GlazeWM.Interprocess.Messages
{
  [Verb(
    "containers",
    HelpText = "Get all containers (monitors, workspaces, windows, split containers)."
  )]
  public class GetContainersMessage
  {
  }
}
