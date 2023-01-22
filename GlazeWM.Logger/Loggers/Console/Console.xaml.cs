using System.Windows;
using System.Windows.Controls;

namespace GlazeWM.Logger.Loggers.Console
{
  /// <summary>
  /// Interaction logic for Console.xaml
  /// </summary>
  public partial class Console : Window
  {
    public readonly RichTextBox LogTextBox;
    public bool CloseWindow;

    public Console()
    {
      InitializeComponent();
      LogTextBox = (RichTextBox)FindName("txt");
      Show();
    }

    private void Window_Closing(object sender, System.ComponentModel.CancelEventArgs e)
    {
      if (!CloseWindow)
        e.Cancel = true;
    }

    private void Window_Closed(object sender, EventArgs e)
    {
      System.Windows.Threading.Dispatcher.CurrentDispatcher.InvokeShutdown();
    }
  }
}
