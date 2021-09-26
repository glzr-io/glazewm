using System;
using System.Collections;
using System.Collections.Generic;
using System.Text;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components
{
  /// <summary>
  /// Interaction logic for ComponentContainer.xaml
  /// </summary>
  public partial class ComponentContainer : UserControl
  {

    public static readonly DependencyProperty ComponentConfigsProperty =
      DependencyProperty.Register("ComponentConfigs", typeof(List<BarComponentConfig>), typeof(ComponentContainer), new FrameworkPropertyMetadata(new List<BarComponentConfig>()));

    public List<BarComponentConfig> ComponentConfigs
    {
      get { return (List<BarComponentConfig>)GetValue(ComponentConfigsProperty); }
      set { SetValue(ComponentConfigsProperty, value); }
    }

    public ComponentContainer()
    {
      InitializeComponent();
    }
  }
}

