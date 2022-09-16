using System;
using System.IO;
using System.Windows.Forms;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Commands;
using Microsoft.Extensions.Options;

namespace GlazeWM.Infrastructure.Exceptions
{
  public class ExceptionHandler
  {
    private readonly Bus _bus;
    private readonly IOptions<ExceptionHandlerOptions> _options;

    public ExceptionHandler(Bus bus, IOptions<ExceptionHandlerOptions> options)
    {
      _bus = bus;
      _options = options;
    }

    public static void HandleNonFatalException(Exception exception)
    {
      // Alert the user of the error.
      MessageBox.Show(exception.Message);
    }

    public void HandleFatalException(Exception exception)
    {
      // Lock bus to prevent further state changes while showing message box and writing to logs.
      lock (_bus.LockObj)
      {
        // Alert the user of the error.
        MessageBox.Show($"Unhandled exception: {exception.Message}");

        WriteToErrorLog(exception);

        _bus.Invoke(new ExitApplicationCommand(true));
      }
    }

    private void WriteToErrorLog(Exception exception)
    {
      var errorLogPath = _options.Value.ErrorLogPath;

      // Create containing directory. Needs to be created before writing to the file.
      Directory.CreateDirectory(Path.GetDirectoryName(errorLogPath));

      File.AppendAllText(
        errorLogPath,
        _options.Value.ErrorLogMessageDelegate(exception)
      );
    }
  }
}
