using LarsWM.Core.Common.Services;
using LarsWM.Core.Monitors;
using LarsWM.Core.Monitors.CommandHandlers;
using LarsWM.Core.Monitors.EventHandler;
using LarsWM.Core.UserConfigs;
using LarsWM.Core.UserConfigs.CommandHandlers;
using LarsWM.Core.Windows;
using LarsWM.Core.Workspaces;
using LarsWM.Core.Workspaces.CommandHandlers;
using Microsoft.Extensions.DependencyInjection;
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace LarsWM.Core
{
    class Program
    {
        public static IServiceProvider ServiceProvider { get; private set; }

        /// <summary>
        ///  The main entry point for the application.
        /// </summary>
        [STAThread]
        static void Main()
        {
            Debug.WriteLine("Application started");

            var serviceCollection = CreateServiceCollection();            

            ServiceProvider = serviceCollection.BuildServiceProvider();

            var bus = ServiceProvider.GetRequiredService<IBus>();
            bus.RegisterCommandHandler<AddMonitorHandler>();
            bus.RegisterCommandHandler<ReadUserConfigHandler>();
            bus.RegisterCommandHandler<CreateWorkspaceHandler>();
            bus.RegisterEventHandler<MonitorAddedHandler>();

            var startup = ServiceProvider.GetRequiredService<Startup>();
            startup.Init();
        }

        private static ServiceCollection CreateServiceCollection()
        {
            var serviceCollection = new ServiceCollection();

            serviceCollection.AddSingleton<IBus, Bus>();
            serviceCollection.AddSingleton<KeybindingService>();
            serviceCollection.AddSingleton<MonitorService>();
            serviceCollection.AddSingleton<UserConfigService>();
            serviceCollection.AddSingleton<WindowService>();
            serviceCollection.AddSingleton<WorkspaceService>();
            serviceCollection.AddSingleton<AddMonitorHandler>();
            serviceCollection.AddSingleton<MonitorAddedHandler>();
            serviceCollection.AddSingleton<ReadUserConfigHandler>();
            serviceCollection.AddSingleton<CreateWorkspaceHandler>();
            serviceCollection.AddSingleton<Startup>();

            return serviceCollection;
        }
    }
}
