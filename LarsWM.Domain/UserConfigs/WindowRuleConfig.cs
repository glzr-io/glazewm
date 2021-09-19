using System;
using System.ComponentModel.DataAnnotations;

namespace LarsWM.Domain.UserConfigs
{
  public class WindowRuleConfig
  {
    public string MatchProcessName { get; set; } = String.Empty;

    public string MatchClassName { get; set; } = String.Empty;

    public string MatchTitle { get; set; } = String.Empty;

    [Required]
    public string Action { get; set; }
  }
}
