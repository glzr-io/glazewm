using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;

namespace LarsWM.Domain.UserConfigs
{
  public class KeybindingConfig
  {
    [Required]
    public string Command { get; set; }
    [Required]
    public List<string> Bindings { get; set; }
  }
}
