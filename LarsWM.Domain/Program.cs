using LarsWM.Domain.Common.Services;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Monitors.CommandHandlers;
using LarsWM.Domain.Monitors.EventHandler;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.UserConfigs.CommandHandlers;
using LarsWM.Domain.Windows;
using LarsWM.Domain.Workspaces;
using LarsWM.Domain.Workspaces.CommandHandlers;
using Microsoft.Extensions.DependencyInjection;
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace LarsWM.Domain
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
