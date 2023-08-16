using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using CommandLine;
using GlazeWM.App.Cli;
using GlazeWM.App.IpcServer;
using GlazeWM.App.IpcServer.Messages;
using GlazeWM.App.Watcher;
using GlazeWM.App.WindowManager;
using GlazeWM.Bar;
using GlazeWM.Domain;
using GlazeWM.Domain.Common;
using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common;
using GlazeWM.Infrastructure.Exceptions;
using GlazeWM.Infrastructure.Logging;
using GlazeWM.Infrastructure.Serialization;
using GlazeWM.Infrastructure.Utils;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Console;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

//// TODO: Handle circular reference for that one workspace event.

namespace GlazeWM.App
{
  internal static class Program
  {
    private const string AppGuid = "325d0ed7-7f60-4925-8d1b-aa287b26b218";
    private const int IpcServerPort = 6123;

    /// <summary>
    /// The main entry point for the application. The thread must be an STA
    /// thread in order to run a message loop.
    /// </summary>
    [STAThread]
    public static async Task<int> Main(string[] args)
    {
      // Allows reading and writing to the console. This is needed since the `OutputType`
      // is `WinExe` and not a console application.
      AttachConsoleToParentProcess();

      bool isSingleInstance;
      using var _ = new Mutex(false, "Global\\" + AppGuid, out isSingleInstance);

      var parsedArgs = Parser.Default.ParseArguments<
        WmStartupOptions,
        WatcherStartupOptions,
        InvokeCommandMessage,
        SubscribeMessage,
        GetMonitorsMessage,
        GetWorkspacesMessage,
        GetWindowsMessage
      >(args);

      var exitCode = parsedArgs.Value switch
      {
        WmStartupOptions options => StartWm(options, isSingleInstance),
        WatcherStartupOptions => await StartWatcher(isSingleInstance),
        InvokeCommandMessage or
        GetMonitorsMessage or
        GetWorkspacesMessage or
        GetWindowsMessage => await StartCli(args, isSingleInstance, false),
        SubscribeMessage => await StartCli(args, isSingleInstance, true),
        _ => ExitWithError(parsedArgs.Errors.First())
      };

      return (int)exitCode;
    }

    private static ExitCode StartWm(WmStartupOptions options, bool isSingleInstance)
    {
      if (!isSingleInstance)
      {
        Console.Error.WriteLine("Application is already running.");
        return ExitCode.Error;
      }

      ServiceLocator.Provider = BuildWmServiceProvider(options);

      var (barService, ipcServerStartup, wmStartup) =
        ServiceLocator.GetRequiredServices<
          BarService,
          IpcServerStartup,
          WmStartup
        >();

      ThreadUtils.CreateSTA("GlazeWMBar", barService.StartApp);
      ThreadUtils.Create("GlazeWMIPC", () => ipcServerStartup.Run(IpcServerPort));

      if (!Debugger.IsAttached)
      {
        var currentExecutable = Process.GetCurrentProcess().MainModule.FileName;
        using var process = new Process();
        process.StartInfo = new ProcessStartInfo
        {
          FileName = currentExecutable,
          Arguments = "watcher",
          UseShellExecute = true,
        };
        process.Start();
      }

      // Run the window manager on the main thread.
      return wmStartup.Run();
    }

    private static async Task<ExitCode> StartCli(
      string[] args,
      bool isSingleInstance,
      bool isSubscribeMessage)
    {
      if (isSingleInstance)
      {
        Console.Error.WriteLine("No running instance found. Cannot run CLI command.");
        return ExitCode.Error;
      }

      return await CliStartup.Run(args, IpcServerPort, isSubscribeMessage);
    }

    private static async Task<ExitCode> StartWatcher(bool isSingleInstance)
    {
      if (isSingleInstance)
      {
        Console.Error.WriteLine("No running instance found. Cannot start watcher.");
        return ExitCode.Error;
      }

      return await new WatcherStartup().Run(IpcServerPort);
    }

    private static ExitCode ExitWithError(Error error)
    {
      Console.Error.WriteLine($"Failed to parse startup arguments: {error}.");
      return ExitCode.Error;
    }

    private static ServiceProvider BuildWmServiceProvider(WmStartupOptions options)
    {
      var services = new ServiceCollection()
        .AddLoggingService()
        .AddExceptionHandler()
        .AddInfrastructureServices()
        .AddDomainServices()
        .AddBarServices()
        .AddIpcServerServices()
        .AddSingleton<WmStartup>()
        .AddSingleton(options);

      return services.BuildServiceProvider();
    }
  }

  public static class DependencyInjection
  {
    public static IServiceCollection AddLoggingService(this IServiceCollection services)
    {
      return services.AddLogging(builder =>
      {
        builder.ClearProviders();
        builder.AddConsoleFormatter<LogFormatter, ConsoleFormatterOptions>();
        builder.AddConsole(options => options.FormatterName = LogFormatter.Name);
        builder.SetMinimumLevel(LogLevel.Debug);
      });
    }

    public static IServiceCollection AddExceptionHandler(this IServiceCollection services)
    {
      services
        .AddOptions<ExceptionHandlingOptions>()
        .Configure<Bus, ContainerService>((options, bus, containerService) =>
        {
          options.ErrorLogPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
            "./.glaze-wm/errors.log"
          );

          options.ErrorLogMessageDelegate = (Exception exception) =>
          {
            var serializeOptions = JsonParser.OptionsFactory(
              options => options.Converters.Add(new JsonContainerConverter())
            );

            var stateDump = JsonParser.ToString(
              containerService.ContainerTree,
              serializeOptions
            );

            // History of latest command invocations. Most recent is first.
            var commandHistory = bus.CommandHistory
              .Select(command => command.Name)
              .Reverse();

            return $"{DateTime.Now}\n"
              + $"{exception}\n"
              + $"Command history: {string.Join(", ", commandHistory)} \n"
              + $"State dump: {stateDump}\n\n";
          };
        });

      return services;
    }
  }
}
