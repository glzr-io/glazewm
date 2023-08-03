using CommandLine;

namespace GlazeWM.Application.IpcServer.Messages
{
  [Verb(
    "subscribe",
    HelpText = "Subscribe to a WM event (eg. `subscribe -e window_focus,window_close`)"
  )]
  public class SubscribeMessage
  {
    [Option(
      'e',
      "events",
      Required = true,
      HelpText = "WM events to subscribe to."
    )]
    public string Events { get; set; }
  }
}
