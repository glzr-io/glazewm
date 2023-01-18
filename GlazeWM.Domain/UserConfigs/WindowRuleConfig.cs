using System;
using System.Collections.Generic;
using System.Text.RegularExpressions;

namespace GlazeWM.Domain.UserConfigs
{
  public class WindowRuleConfig
  {
    public string MatchProcessName { get; set; }

    public Regex ProcessNameRegex => CreateRegex(MatchProcessName);

    public string MatchClassName { get; set; }

    public Regex ClassNameRegex => CreateRegex(MatchClassName);

    public string MatchTitle { get; set; }

    public Regex TitleRegex => CreateRegex(MatchTitle);

    public string Command { get; set; }

    public List<string> Commands { get; set; } = new List<string>();

    public List<string> CommandList =>
      Command != null ? new List<string> { Command } : Commands;

    /// <summary>
    /// Creates an exact match regex for the given string.
    /// </summary>
    /// <returns>The corresponding regex, or null if input is invalid.</returns>
    private static Regex CreateRegex(string input)
    {
      if (string.IsNullOrWhiteSpace(input))
        return null;

      var isRegexLiteral =
        input.StartsWith("/", StringComparison.InvariantCulture) &&
        input.EndsWith("/", StringComparison.InvariantCulture);

      // Allow user to pass a string that should be interpreted as regex (eg. "/steam/").
      if (isRegexLiteral)
        return new Regex(input[1..^1]);

      // Otherwise, create an exact match regex.
      return new Regex($"^{input}$");
    }
  }
}
