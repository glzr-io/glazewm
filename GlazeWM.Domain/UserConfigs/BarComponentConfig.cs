using System.ComponentModel.DataAnnotations;

namespace GlazeWM.Domain.UserConfigs
{
  public class BarComponentConfig : CommonBarAttributes
  {
    [Required]
    public string Type { get; set; }

    public string Margin { get; set; } = "0 10 0 0";
  }
}
