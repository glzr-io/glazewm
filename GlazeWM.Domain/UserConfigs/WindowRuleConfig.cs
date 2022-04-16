using System;
using System.Collections.Generic;
using System.Text.RegularExpressions;

namespace GlazeWM.Domain.UserConfigs
{
  public class WindowRuleConfig
  {
    public string MatchProcessName { get; set; } = null;

    public Regex ProcessNameRegex => CreateRegex(MatchProcessName);

    public string MatchClassName { get; set; } = null;

    public Regex ClassNameRegex => CreateRegex(MatchClassName);

    public string MatchTitle { get; set; } = null;

    public Regex TitleRegex => CreateRegex(MatchTitle);

    public string Command { get; set; } = null;

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

      var isRegexLiteral = input.StartsWith("/") && input.EndsWith("/");

      // Allow user to pass a string that should be interpreted as regex (eg. "/steam/").
      if (isRegexLiteral)
        return new Regex(input.Substring(1, input.Length - 2));

      // Otherwise, create an exact match regex.
      return new Regex($"^{input}$");
    }
  }
}
