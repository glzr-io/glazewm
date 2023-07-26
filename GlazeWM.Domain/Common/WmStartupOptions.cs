using CommandLine;

namespace GlazeWM.Domain.Common
{
  public class WmStartupOptions
  {
    [Option(
      'c',
      "config",
      Required = false,
      HelpText = "Custom path to user config file."
    )]
    public string ConfigPath { get; set; }
  }
}
