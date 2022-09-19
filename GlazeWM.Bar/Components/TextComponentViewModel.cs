using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  public class TextComponentViewModel : ComponentViewModel
  {
    private TextComponentConfig _config => _componentConfig as TextComponentConfig;

    public string Text => _config.Text;

    public TextComponentViewModel(
      BarViewModel parentViewModel,
      TextComponentConfig config) : base(parentViewModel, config)
    {
    }
  }
}
