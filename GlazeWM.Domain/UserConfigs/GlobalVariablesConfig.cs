using System.ComponentModel.DataAnnotations;

namespace GlazeWM.Domain.UserConfigs
{
  public class GlobalVariablesConfig
  {
    [Required]
    public string Name { get; set; }

    [Required]
    public string Value { get; set; }
  }
}
