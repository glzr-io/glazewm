using System.ComponentModel.DataAnnotations;

namespace GlazeWM.Domain.UserConfigs
{
  public class WorkspaceConfig
  {
    [Required]
    public string Name { get; set; }
    public string BindToMonitor { get; set; }
    public string DisplayName { get; set; }
    public bool KeepAlive { get; set; }
  }
}
