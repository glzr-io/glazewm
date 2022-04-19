using System.ComponentModel.DataAnnotations;

namespace GlazeWM.Domain.UserConfigs
{
  public class WorkspaceConfig
  {
    [Required]
    public string Name { get; set; }
    public string BindToMonitor { get; set; } = null;
    public string DisplayName { get; set; } = null;
    public bool KeepAlive { get; set; } = false;
  }
}
