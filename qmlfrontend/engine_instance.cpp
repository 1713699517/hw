#include "engine_instance.h"

extern "C" void (*getProcAddress())(const char* fn) { return nullptr; }

EngineInstance::EngineInstance(QObject* parent)
    : QObject(parent), m_instance(Engine::start_engine()) {}

EngineInstance::~EngineInstance() { Engine::cleanup(m_instance); }

void EngineInstance::sendConfig(const GameConfig& config) {
  for (auto b : config.config()) {
    Engine::send_ipc(m_instance, reinterpret_cast<uint8_t*>(b.data()),
                     static_cast<size_t>(b.size()));
  }
}

void EngineInstance::advance(quint32 ticks) {}

void EngineInstance::renderFrame() {}

void EngineInstance::setOpenGLContext(QOpenGLContext* context) {
  Engine::setup_current_gl_context(m_instance, 0, 0, &getProcAddress);
}

Engine::PreviewInfo EngineInstance::generatePreview() {
  Engine::PreviewInfo pinfo;

  Engine::generate_preview(m_instance, &pinfo);

  return pinfo;
}
