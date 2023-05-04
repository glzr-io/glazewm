using System.Windows.Input;
using GlazeWM.Bar.Common;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Bar.Components
{
  public class BarSeperatorComponentViewModel : ComponentViewModel
  {
    private readonly Bus _bus = ServiceLocator.GetRequiredService<Bus>();
    private readonly ContainerService _containerService =
      ServiceLocator.GetRequiredService<ContainerService>();
    private readonly CommandParsingService _commandParsingService =
      ServiceLocator.GetRequiredService<CommandParsingService>();
    private BarSeperatorComponentConfig _config => _componentConfig as BarSeperatorComponentConfig;

    public string Text => _config.Text;

    public ICommand LeftClickCommand => new RelayCommand(OnLeftClick);
    public ICommand RightClickCommand => new RelayCommand(OnRightClick);

    public BarSeperatorComponentViewModel(
      BarViewModel parentViewModel,
      BarSeperatorComponentConfig config) : base(parentViewModel, config)
    {
    }

    public void OnLeftClick()
    {
      InvokeCommand(_config.LeftClickCommand);
    }

    public void OnRightClick()
    {
      InvokeCommand(_config.RightClickCommand);
    }

    private void InvokeCommand(string commandString)
    {
      if (string.IsNullOrEmpty(commandString))
        return;

      var subjectContainer = _containerService.FocusedContainer;

      var parsedCommand = _commandParsingService.ParseCommand(
        commandString,
        subjectContainer
      );

      // Use `dynamic` to resolve the command type at runtime and allow multiple dispatch.
      _bus.Invoke((dynamic)parsedCommand);
    }
  }
}
