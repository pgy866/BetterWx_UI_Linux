#include <napi.h>
#include <node_api.h>
#include <app-indicator.h>
#include <gtk/gtk.h>
#include <dbus/dbus-glib.h>

class LinuxTray : public Napi::ObjectWrap<LinuxTray> {
public:
    static Napi::Object Init(Napi::Env env, Napi::Object exports);
    LinuxTray(const Napi::CallbackInfo& info);
    ~LinuxTray();

private:
    AppIndicator* indicator;
    GtkWidget* menu;
    Napi::ThreadSafeFunction onClickCallback;

    static void OnMenuClick(GtkMenuItem* item, gpointer data);
    void CreateIndicator(const char* iconPath);
    void BuildMenu(Napi::Array items);
    Napi::Value SetIcon(const Napi::CallbackInfo& info);
    Napi::Value UpdateMenu(const Napi::CallbackInfo& info);
};

Napi::Object LinuxTray::Init(Napi::Env env, Napi::Object exports) {
    Napi::Function func = DefineClass(env, "LinuxTray", {
        InstanceMethod("setIcon", &LinuxTray::SetIcon),
        InstanceMethod("updateMenu", &LinuxTray::UpdateMenu)
    });

    exports.Set("LinuxTray", func);
    return exports;
}

LinuxTray::LinuxTray(const Napi::CallbackInfo& info) : Napi::ObjectWrap<LinuxTray>(info) {
    Napi::Env env = info.Env();
    
    // 初始化GTK线程
    if (!g_thread_supported()) g_thread_init(NULL);
    gdk_threads_init();
    gdk_threads_enter();
    gtk_init_check(NULL, NULL);

    // 解析参数
    std::string iconPath = info[0].As<Napi::String>();
    Napi::Array menuItems = info[1].As<Napi::Array>();
    this->onClickCallback = Napi::ThreadSafeFunction::New(
        env,
        info[2].As<Napi::Function>(),
        "TrayClickHandler",
        0,
        1
    );

    // 创建系统托盘
    this->CreateIndicator(iconPath.c_str());
    this->BuildMenu(menuItems);

    // 注册DBus服务
    dbus_g_connection_register_g_main(dbus_g_bus_get(DBUS_BUS_SESSION, NULL), NULL);
    gtk_timeout_add(100, (GSourceFunc)gtk_main_iteration_do, FALSE);
    gdk_threads_leave();
}

void LinuxTray::CreateIndicator(const char* iconPath) {
    this->indicator = app_indicator_new(
        "betterwx-indicator",
        iconPath,
        APP_INDICATOR_CATEGORY_APPLICATION_STATUS
    );
    
    app_indicator_set_status(this->indicator, APP_INDICATOR_STATUS_ACTIVE);
    app_indicator_set_attention_icon(this->indicator, "indicator-messages-new");
}

void LinuxTray::BuildMenu(Napi::Array items) {
    this->menu = gtk_menu_new();
    
    for (uint32_t i = 0; i < items.Length(); ++i) {
        Napi::Object item = items.Get(i).As<Napi::Object>();
        std::string label = item.Get("label").As<Napi::String>();
        std::string type = item.Get("type").As<Napi::String>();
        
        if (type == "separator") {
            GtkWidget* sep = gtk_separator_menu_item_new();
            gtk_menu_shell_append(GTK_MENU_SHELL(this->menu), sep);
        } else {
            GtkWidget* menuItem = gtk_menu_item_new_with_label(label.c_str());
            g_signal_connect(menuItem, "activate", G_CALLBACK(OnMenuClick), this);
            gtk_menu_shell_append(GTK_MENU_SHELL(this->menu), menuItem);
        }
    }
    
    app_indicator_set_menu(this->indicator, GTK_MENU(this->menu));
    gtk_widget_show_all(this->menu);
}

void LinuxTray::OnMenuClick(GtkMenuItem* item, gpointer data) {
    LinuxTray* self = static_cast<LinuxTray*>(data);
    gchar* label = (gchar*)gtk_menu_item_get_label(item);
    
    self->onClickCallback.Acquire()->BlockingCall([label](Napi::Env env, Napi::Function jsCallback) {
        jsCallback.Call({ Napi::String::New(env, label) });
    });
}

Napi::Value LinuxTray::SetIcon(const Napi::CallbackInfo& info) {
    std::string iconPath = info[0].As<Napi::String>();
    
    gdk_threads_enter();
    app_indicator_set_icon(this->indicator, iconPath.c_str());
    gdk_threads_leave();
    
    return info.Env().Undefined();
}

Napi::Value LinuxTray::UpdateMenu(const Napi::CallbackInfo& info) {
    Napi::Array newItems = info[0].As<Napi::Array>();
    
    gdk_threads_enter();
    gtk_container_foreach(GTK_CONTAINER(this->menu), 
        [](GtkWidget* child, gpointer data) {
            gtk_widget_destroy(child);
        }, NULL);
    
    this->BuildMenu(newItems);
    gdk_threads_leave();
    
    return info.Env().Undefined();
}

// 模块注册
Napi::Object Init(Napi::Env env, Napi::Object exports) {
    return LinuxTray::Init(env, exports);
}

NODE_API_MODULE(NODE_GYP_MODULE_NAME, Init)