using CommandLine;

namespace GlazeWM.Interprocess.Messages
{
  [Verb(
    "commit",
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
    public bool Events { get; set; }
  }
}
