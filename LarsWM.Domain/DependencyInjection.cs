using LarsWM.Domain.Common.Services;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.CommandHandlers;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Monitors.CommandHandlers;
using LarsWM.Domain.Monitors.EventHandler;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.UserConfigs.CommandHandlers;
using LarsWM.Domain.Windows;
using LarsWM.Domain.Windows.CommandHandlers;
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
            services.AddSingleton<ContainerService>();
            services.AddSingleton<MonitorService>();
            services.AddSingleton<UserConfigService>();
            services.AddSingleton<WindowService>();
            services.AddSingleton<WindowHooksHandler>();
            services.AddSingleton<WorkspaceService>();
            services.AddSingleton<AttachContainerHandler>();
            services.AddSingleton<DetachContainerHandler>();
            services.AddSingleton<RedrawContainersHandler>();
            services.AddSingleton<AddMonitorHandler>();
            services.AddSingleton<AttachWorkspaceToMonitorHandler>();
            services.AddSingleton<EvaluateUserConfigHandler>();
            services.AddSingleton<AddInitialWindowsHandler>();
            services.AddSingleton<AddWindowHandler>();
            services.AddSingleton<FocusWindowHandler>();
            services.AddSingleton<CreateWorkspaceHandler>();
            services.AddSingleton<DisplayWorkspaceHandler>();
            services.AddSingleton<FocusWorkspaceHandler>();
            services.AddSingleton<MonitorAddedHandler>();

            return services;
        }

        public static IServiceProvider RegisterDomainHandlers(this IServiceProvider serviceProvider)
        {
            var bus = serviceProvider.GetRequiredService<IBus>();
            bus.RegisterCommandHandler<AttachContainerHandler>();
            bus.RegisterCommandHandler<DetachContainerHandler>();
            bus.RegisterCommandHandler<RedrawContainersHandler>();
            bus.RegisterCommandHandler<AddMonitorHandler>();
            bus.RegisterCommandHandler<AttachWorkspaceToMonitorHandler>();
            bus.RegisterCommandHandler<EvaluateUserConfigHandler>();
            bus.RegisterCommandHandler<AddInitialWindowsHandler>();
            bus.RegisterCommandHandler<AddWindowHandler>();
            bus.RegisterCommandHandler<FocusWindowHandler>();
            bus.RegisterCommandHandler<CreateWorkspaceHandler>();
            bus.RegisterCommandHandler<DisplayWorkspaceHandler>();
            bus.RegisterCommandHandler<FocusWorkspaceHandler>();
            bus.RegisterEventHandler<MonitorAddedHandler>();

            return serviceProvider;
        }
    }
}
