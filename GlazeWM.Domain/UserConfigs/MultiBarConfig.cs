namespace GlazeWM.Domain.UserConfigs
{
  public class MultiBarConfig : BarConfig
  {
    // TODO: Put this logic somewhere shared, so that it's not repeated in
    // `WorkspaceConfig.BindToMonitor`.
    private string _bindToMonitor;
    public string BindToMonitor
    {
      get => _bindToMonitor;
      set => _bindToMonitor = int.TryParse(value, out var monitorIndex)
        ? $@"\\.\DISPLAY{monitorIndex}"
        : value;
    }
  }
}
