using LarsWM.Domain.Common.Services;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Monitors.CommandHandlers;
using LarsWM.Domain.Monitors.EventHandler;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.UserConfigs.CommandHandlers;
using LarsWM.Domain.Windows;
using LarsWM.Domain.Workspaces;
using LarsWM.Domain.Workspaces.CommandHandlers;
using LarsWM.Infrastructure.Bussing;
using Microsoft.Extensions.DependencyInjection;
using System;

namespace LarsWM.Domain
{
    public static class DependencyInjection
    {
        public static IServiceCollection AddDomainServices(this IServiceCollection services)
        {
            services.AddSingleton<KeybindingService>();
            services.AddSingleton<MonitorService>();
            services.AddSingleton<UserConfigService>();
            services.AddSingleton<WindowService>();
            services.AddSingleton<WorkspaceService>();
            services.AddSingleton<AddMonitorHandler>();
            services.AddSingleton<MonitorAddedHandler>();
            services.AddSingleton<ReadUserConfigHandler>();
            services.AddSingleton<CreateWorkspaceHandler>();

            return services;
        }

        public static IServiceProvider RegisterDomainHandlers(this IServiceProvider serviceProvider)
        {
            var bus = serviceProvider.GetRequiredService<IBus>();
            bus.RegisterCommandHandler<AddMonitorHandler>();
            bus.RegisterCommandHandler<ReadUserConfigHandler>();
            bus.RegisterCommandHandler<CreateWorkspaceHandler>();
            bus.RegisterEventHandler<MonitorAddedHandler>();

            return serviceProvider;
        }
    }
}
