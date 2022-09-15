using System;
using System.IO;
using System.Windows.Forms;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Commands;

namespace GlazeWM.Infrastructure.Exceptions
{
  public class ExceptionHandler
  {
    private readonly Bus _bus;

    public ExceptionHandler(Bus bus)
    {
      _bus = bus;
    }

    public static void HandleNonFatalException(Exception exception)
    {
      MessageBox.Show(exception.Message);
    }

    public void HandleFatalException(Exception exception)
    {
      // Alert the user of the error.
      if (exception is FatalUserException)
        MessageBox.Show(exception.Message);

      WriteToErrorLog(exception);

      _bus.Invoke(new ExitApplicationCommand(true));
    }

    private static void WriteToErrorLog(Exception exception)
    {
      var errorLogPath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
        "./.glaze-wm/errors.log"
      );

      Directory.CreateDirectory(Path.GetDirectoryName(errorLogPath));

      File.AppendAllText(
        errorLogPath,
        $"\n\n{DateTime.Now}\n{exception.Message + exception.ToString()}"
      );
    }
  }
}
