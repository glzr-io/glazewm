using System;
using System.ComponentModel.DataAnnotations;

namespace GlazeWM.Domain.UserConfigs
{
  public class WorkspaceConfig
  {
    [Required]
    public string Name { get; set; }

    private string _bindToMonitor;
    public string BindToMonitor
    {
      get => int.TryParse(_bindToMonitor, out var monitorIndex) ? $@"\\.\DISPLAY{monitorIndex}" : _bindToMonitor;
      set => _bindToMonitor = value;
    }

    public string DisplayName { get; set; }
    public bool KeepAlive { get; set; }
  }
}
