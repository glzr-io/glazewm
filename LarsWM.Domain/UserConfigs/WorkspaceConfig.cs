using System.ComponentModel.DataAnnotations;

namespace LarsWM.Domain.UserConfigs
{
  public class WorkspaceConfig
  {
    [Required]
    public string Name { get; set; }
    public string BindToMonitor { get; set; } = null;
    public string CustomDisplayName { get; set; } = null;
    public bool KeepAlive { get; set; } = false;
  }
}
