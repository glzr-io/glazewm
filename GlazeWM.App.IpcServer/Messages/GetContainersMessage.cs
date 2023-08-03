using CommandLine;

namespace GlazeWM.App.IpcServer.Messages
{
  [Verb(
    "containers",
    HelpText = "Get all containers (monitors, workspaces, windows, split containers)."
  )]
  public class GetContainersMessage
  {
  }
}
