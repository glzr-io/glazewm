using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.Text.RegularExpressions;

namespace LarsWM.Domain.UserConfigs
{
  public class WindowRuleConfig
  {
    public string MatchProcessName { get; set; } = null;

    public Regex ProcessNameRegex
    {
      get
      {
        if (MatchProcessName == null || MatchProcessName.Trim() == String.Empty)
          return new Regex(".*");

        var startIndex = MatchProcessName.IndexOf("/") + "/".Length;
        var endIndex = MatchProcessName.LastIndexOf("/");

        var pattern = MatchProcessName.Substring(startIndex, endIndex - startIndex);

        if (pattern != null)
          return new Regex(pattern);

        return new Regex(MatchProcessName);
      }
    }

    public string MatchClassName { get; set; } = null;

    public Regex ClassNameRegex
    {
      get
      {
        if (MatchClassName == null || MatchClassName.Trim() == String.Empty)
          return new Regex(".*");

        var startIndex = MatchClassName.IndexOf("/") + "/".Length;
        var endIndex = MatchClassName.LastIndexOf("/");

        var pattern = MatchClassName.Substring(startIndex, endIndex - startIndex);

        if (pattern != null)
          return new Regex(pattern);

        return new Regex(MatchClassName);
      }
    }

    public string MatchTitle { get; set; } = null;

    public Regex TitleRegex
    {
      get
      {
        if (MatchTitle == null || MatchTitle.Trim() == String.Empty)
          return new Regex(".*");

        var startIndex = MatchTitle.IndexOf("/") + "/".Length;
        var endIndex = MatchTitle.LastIndexOf("/");

        var pattern = MatchTitle.Substring(startIndex, endIndex - startIndex);

        if (pattern != null)
          return new Regex(pattern);

        return new Regex(MatchTitle);
      }
    }

    public string Action { get; set; } = null;

    public List<string> Actions { get; set; } = new List<string>();
  }
}
