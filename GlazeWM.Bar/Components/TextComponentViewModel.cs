using System.Windows.Input;
using GlazeWM.Bar.Common;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Bar.Components
{
  public class TextComponentViewModel : ComponentViewModel
  {
    private readonly Bus _bus = ServiceLocator.Provider.GetRequiredService<Bus>();
    private readonly ContainerService _containerService =
      ServiceLocator.Provider.GetRequiredService<ContainerService>();
    private readonly CommandParsingService _commandParsingService =
      ServiceLocator.Provider.GetRequiredService<CommandParsingService>();
    private TextComponentConfig _config => _componentConfig as TextComponentConfig;

    public string Text => _config.Text;

    public ICommand LeftClickCommand => new RelayCommand(OnLeftClick);
    public ICommand RightClickCommand => new RelayCommand(OnRightClick);

    public TextComponentViewModel(
      BarViewModel parentViewModel,
      TextComponentConfig config) : base(parentViewModel, config)
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
