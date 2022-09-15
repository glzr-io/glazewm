using System;
using System.Globalization;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  public class TextComponentViewModel : ComponentViewModel
  {
    private TextComponentConfig _config => _componentConfig as TextComponentConfig;
    private string _text => _config.Text;

    /// <summary>
    /// Shows text from the user's config.
    /// </summary>
    public string Text => _text;

    public TextComponentViewModel(BarViewModel parentViewModel, TextComponentConfig config) : base(parentViewModel, config)
    {

      
      // Update the displayed time every second.
      // This is useless for this component
      // since it's a fixed text.
      var updateInterval = TimeSpan.FromSeconds(1);

      Observable.Interval(updateInterval)
        .Subscribe(_ => OnPropertyChanged(nameof(Text)));
    }
  }
}
