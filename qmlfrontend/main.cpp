#include <QDebug>
#include <QGuiApplication>
#include <QLibrary>
#include <QQmlApplicationEngine>

#include "engine_interface.h"
#include "hwengine.h"

namespace Engine {
protocol_version_t* protocol_version;
start_engine_t* start_engine;
generate_preview_t* generate_preview;
cleanup_t* cleanup;
};  // namespace Engine

void loadEngineLibrary() {
#ifdef Q_OS_WIN
  QLibrary hwlib("./libhedgewars_engine.dll");
#else
  QLibrary hwlib("./libhedgewars_engine.so");
#endif

  if (!hwlib.load())
    qWarning() << "Engine library not found" << hwlib.errorString();

  Engine::protocol_version = reinterpret_cast<Engine::protocol_version_t*>(
      hwlib.resolve("protocol_version"));
  Engine::start_engine =
      reinterpret_cast<Engine::start_engine_t*>(hwlib.resolve("start_engine"));
  Engine::generate_preview = reinterpret_cast<Engine::generate_preview_t*>(
      hwlib.resolve("generate_preview"));
  Engine::cleanup =
      reinterpret_cast<Engine::cleanup_t*>(hwlib.resolve("cleanup"));

  if (Engine::protocol_version)
    qDebug() << "Loaded engine library with protocol version"
             << Engine::protocol_version();
}

int main(int argc, char* argv[]) {
  QCoreApplication::setAttribute(Qt::AA_EnableHighDpiScaling);
  QGuiApplication app(argc, argv);

  loadEngineLibrary();

  QQmlApplicationEngine engine;

  HWEngine::exposeToQML();

  engine.load(QUrl(QLatin1String("qrc:/main.qml")));
  if (engine.rootObjects().isEmpty()) return -1;

  return app.exec();
}
