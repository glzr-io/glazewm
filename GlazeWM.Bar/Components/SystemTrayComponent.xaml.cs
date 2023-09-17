using System.Windows.Controls;
using System.Windows.Input;

namespace GlazeWM.Bar.Components
{
  /// <summary>
  /// Interaction logic for SystemTrayComponent.xaml
  /// </summary>
  public partial class SystemTrayComponent : UserControl
  {
    public SystemTrayComponent()
    {
      InitializeComponent();
    }

    public void OnLabelHoverEnter(object sender, MouseEventArgs e)
    {
      Cursor = Cursors.Hand;
    }

    public void OnLabelHoverLeave(object sender, MouseEventArgs e)
    {
      Cursor = Cursors.Arrow;
    }
  }
}
