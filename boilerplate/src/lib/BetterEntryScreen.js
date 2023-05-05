/*
 * BetterEntryScreen by muno9748
 * https://github.com/muno9748/BetterEntryScreen
 * MIT LICENSE
 */

/*
 * Modified
 */

window.EntryScreenFixerWebGL = class EntryScreenFixerWebGL {
    static SVGMarker = Symbol('SVGMarker')

    constructor(parent, Entry) {
        this.parent = parent
        this.Entry = Entry
    }
  
    ratioX = 0
    ratioY = 0

    fixEntity(objMap, ett) {
        const Entry = this.Entry
        const originSetImage = ett.setImage.bind(ett)
        const originSetScaleX = ett.setScaleX.bind(ett)
        const originSetScaleY = ett.setScaleY.bind(ett)

        let sx = ett.scaleX
        let sy = ett.scaleY

        ett.setImage = pictureModel => {
            const idx = ett.parent.pictures.indexOf(pictureModel)

            if(objMap.has(idx)) {
                const data = objMap.get(idx)

                delete pictureModel._id
                Entry.assert(ett.type == 'sprite', 'Set image is only for sprite object')
            
                if (!pictureModel.id) pictureModel.id = Entry.generateHash()
            
                ett.picture = pictureModel

                const entityWidth = ett.getWidth()
                const entityHeight = ett.getHeight()
                const absoluteRegX = ett.getRegX() - entityWidth / 2
                const absoluteRegY = ett.getRegY() - entityHeight / 2

                ett.object.texture = data.texture
                originSetScaleX(sx / data.ratioX)
                originSetScaleY(sy / data.ratioY)
                ett.setRegX(data.regX + absoluteRegX)
                ett.setRegY(data.regY + absoluteRegY)
                ett.setWidth(data.regX * 2)
                ett.setHeight(data.regY * 2)

                ett._scaleAdaptor.updateScaleFactor()
        
                ett.object.refreshFilter()
            
                Entry.dispatchEvent('updateObject')
            } else {
                originSetImage(pictureModel)
            }
        }

        ett.setScaleX = scaleX => {
            ett.scaleX = scaleX
        
            ett._scaleAdaptor.scale.setX(scaleX)
    
            if (ett.textObject) ett.textObject.setFontScaleX(scaleX)
        
            ett.parent.updateCoordinateView()
            ett.updateDialog()

            Entry.requestUpdate = true

            const idx = ett.parent.pictures.indexOf(ett.picture)

            if(objMap.has(idx)) {
                const data = objMap.get(idx)

                sx = scaleX * data.factorX
            }
        }

        ett.setScaleY = scaleY => {
            ett.scaleY = scaleY
        
            ett._scaleAdaptor.scale.setY(scaleY)
    
            if (ett.textObject) ett.textObject.setFontScaleY(scaleY)
        
            ett.parent.updateCoordinateView()
            ett.updateDialog()

            Entry.requestUpdate = true

            const idx = ett.parent.pictures.indexOf(ett.picture)

            if(objMap.has(idx)) {
                const data = objMap.get(idx)

                sy = scaleY * data.factorY
            }
        }

        ett.setScaleX(ett.scaleX)
        ett.setScaleY(ett.scaleY)
        ett.setImage(ett.picture)
    }

    fixEntries() {
        const { Entry } = this

        let entries = 0
        let pngEntries = 0

        Entry.container.objects_.forEach(obj => {
            if(obj.esf_marker == EntryScreenFixer.Marker) return
            if(obj.objectType != 'sprite') return
            
            const objMap = new Map()

            obj.esf_marker = EntryScreenFixer.Marker

            const originalImage = obj.entity.picture

            obj.pictures.forEach((pic, idx) => {
                if(pic.imageType == 'svg' && !pic.fileurl) {
                    obj.entity.setImage(pic)

                    const id = pic.filename
                    const src = `./project/temp/${id.slice(0, 2)}/${id.slice(2, 4)}/image/${id}.svg`
                    const texture = obj.entity.object.texture.constructor.from(src)

                    const t = obj.entity.object.texture
                    const w = obj.entity.object.texture.width
                    const h = obj.entity.object.texture.height

                    texture.baseTexture.resource.load().then(() => {
                        texture.textureScaleFactorX = 1
                        texture.textureScaleFactorY = 1

                        objMap.set(idx, {
                            texture,
                            regX: texture.width / 2,
                            regY: texture.height / 2,
                            ratioX: texture.width / w,
                            ratioY: texture.height / h,
                            factorX: t.textureScaleFactorX,
                            factorY: t.textureScaleFactorY,
                        })
                    })
                    
                    entries++
                } else if (pic.imageType == 'png' && !pic.fileurl) {
                    obj.entity.setImage(pic)

                    const id = pic.filename
                    const src = `./project/temp/${id.slice(0, 2)}/${id.slice(2, 4)}/image/${id}.png`
                    const texture = obj.entity.object.texture.constructor.from(src)

                    const t = obj.entity.object.texture
                    const w = obj.entity.object.texture.width
                    const h = obj.entity.object.texture.height

                    texture.baseTexture.resource.load().then(() => {
                        texture.textureScaleFactorX = 1
                        texture.textureScaleFactorY = 1

                        objMap.set(idx, {
                            texture,
                            regX: texture.width / 2,
                            regY: texture.height / 2,
                            ratioX: texture.width / w,
                            ratioY: texture.height / h,
                            factorX: t.textureScaleFactorX,
                            factorY: t.textureScaleFactorY,
                        })
                    })

                    pngEntries++
                }
            })

            obj.entity.setImage(originalImage)

            this.fixEntity(objMap, obj.entity)
    
            obj.addCloneEntity = (_object, entity, _script) => {
                if (obj.clonedEntities.length > Entry.maxCloneLimit) return
        
                const clonedEntity = new Entry.EntityObject(obj)
                
                clonedEntity.isClone = true

                entity = entity || obj.entity
                
                clonedEntity.injectModel(entity.picture || null, entity.toJSON())
                clonedEntity.snapshot_ = entity.snapshot_
        
                if (entity.effect) {
                    clonedEntity.effect = _.clone(entity.effect)
                    clonedEntity.applyFilter()
                }
        
                Entry.engine.raiseEventOnEntity(clonedEntity, [clonedEntity, 'when_clone_start'])

                clonedEntity.isStarted = true

                obj.addCloneVariables(obj, clonedEntity, entity ? entity.variables : null, entity ? entity.lists : null)
                obj.clonedEntities.push(clonedEntity)

                let targetIndex = Entry.stage.selectedObjectContainer.getChildIndex(entity.object)
                targetIndex -= (entity.shapes.length ? 1 : 0) + entity.stamps.length

                Entry.stage.loadEntity(clonedEntity, targetIndex)
        
                if (entity.brush) Entry.setCloneBrush(clonedEntity, entity.brush)

                this.fixEntity(objMap, clonedEntity)
            }
        })

        this.parent.info(`Fixed ${entries} SVG Entries and ${pngEntries} Cached Entries`)
    }

    fix() {
        const resolution = [Math.ceil(screen.width), Math.ceil(screen.width * 9 / 16)]

        this.parent.info(`Found Device Resolution ${screen.width}x${screen.height}`)

        this.fixEntries()

        this.setScreenResolution(...resolution)
    }

    setScreenResolution(w, h) {
        this.parent.info(`Setting Rendering Resolution to ${w}x${h}`)

        const { Entry } = this
        const { stage } = Entry
    
        stage.canvas.canvas.width = w
        stage.canvas.canvas.height = h
        stage.canvas.x = w / 2
        stage.canvas.y = h / 2
        stage.canvas.scaleX = stage.canvas.scaleY = w / 240 / 2
    
        stage._app.screen.width = w
        stage._app.screen.height = h

        this.ratioX = 480 / w
        this.ratioY = 270 / h

        Entry.requestUpdate = true

        this.fixRatio()
    }

    fixRatio() {
        const { Entry } = this

        Entry.variableContainer.lists_.forEach(list => {
            list.view_.off('__pointerdown')
            list.view_.off('__pointermove')
            list.view_.off('__pointerup')
            list.view_.off('pointerover')
            list.resizeHandle_.off('__pointerdown')
            list.resizeHandle_.off('__pointermove')
            list.resizeHandle_.off('__pointerup')
            list.resizeHandle_.off('pointerover')
            list.scrollButton_.off('__pointerdown')
            list.scrollButton_.off('__pointermove')
            list.scrollButton_.off('__pointerup')
            list.scrollButton_.off('pointerover')

            list.view_.on('pointerover', () => list.view_.cursor = 'move')
            list.view_.on('__pointermove', e => {
                if (Entry.type != 'workspace' || list.isResizing) return

                list.view_.offset = {
                    x: list.view_.x - (e.stageX * this.ratioX - 240),
                    y: list.view_.y - (e.stageY * this.ratioY - 135)
                }

                list.view_.cursor = 'move'
            })
            list.view_.on('__pointerdown', () => {
                list.view_.cursor = 'initial'
                list.isResizing = false
            })
            list.view_.on('__pointerup', e => {
                if (Entry.type != 'workspace' || list.isResizing) return

                list.setX(e.stageX * this.ratioX - 240 + list.view_.offset.x)
                list.setY(e.stageY * this.ratioY - 135 + list.view_.offset.y)
                list.updateView()
            })

            list.resizeHandle_.on('pointerover', () => list.resizeHandle_.cursor = 'nwse-resize')
            list.resizeHandle_.on('__pointermove', e => {
                list.isResizing = true
            
                list.resizeHandle_.offset = {
                    x: e.stageX * this.ratioX - list.getWidth(),
                    y: e.stageY * this.ratioY - list.getHeight()
                }

                list.view_.cursor = 'nwse-resize'
            })
            list.resizeHandle_.on('__pointerup', e => {
                list.setWidth(e.stageX * this.ratioX - list.resizeHandle_.offset.x)
                list.setHeight(e.stageY * this.ratioY - list.resizeHandle_.offset.y)
                list.updateView()
            })

            list.scrollButton_.on('__pointermove', e => {
                list.isResizing = true
                list.scrollButton_.offsetY = e.stageY - list.scrollButton_.y / this.ratioY
            })
            list.scrollButton_.on('__pointerup', e => {
                list.scrollButton_.y = Math.min(Math.max((e.stageY - list.scrollButton_.offsetY) * this.ratioY, 25), list.getHeight() - 30)
                list.updateView()
            })
        })

        Entry.variableContainer.variables_.forEach(v => {
            v.view_.off('__pointerdown')
            v.view_.off('__pointermove')
            v.view_.off('__pointerup')
            v.view_.off('pointerover')

            v.view_.on('__pointermove', e => {
                if (Entry.type != 'workspace') return

                v.view_.offset = {
                    x: v.view_.x - (e.stageX * this.ratioX - 240),
                    y: v.view_.y - (e.stageY * this.ratioY - 135)
                }
            })

            v.view_.on('__pointerup', e => {
                if (Entry.type != 'workspace') return

                v.setX(e.stageX * this.ratioX - 240 + v.view_.offset.x)
                v.setY(e.stageY * this.ratioY - 135 + v.view_.offset.y)
                v.updateView()
            })

            if(!v.slideBar_) return

            v.slideBar_.off('__pointerdown')
            v.slideBar_.off('__pointermove')
            v.slideBar_.off('__pointerup')
            v.slideBar_.off('pointerover')
            v.valueSetter_.off('__pointerdown')
            v.valueSetter_.off('__pointermove')
            v.valueSetter_.off('__pointerup')
            v.valueSetter_.off('pointerover')
            
            v.slideBar_.on('__pointermove', e => {
                if (!Entry.engine.isState('run')) return

                const value = evt.stageX * this.ratioX - (v.slideBar_.getX() + 240 + 5) + 5

                v.setSlideCommandX(value)
            })

            v.valueSetter_.on('__pointermove', e => {
                if (!Entry.engine.isState('run')) return

                v.isAdjusting = true
                v.valueSetter_.offsetX = e.stageX * this.ratioX - v.valueSetter_.x
            })
    
            v.valueSetter_.on('__pointerup', e => {
                if (!Entry.engine.isState('run')) return

                const value = (e.stageX * this.ratioX) - v.valueSetter_.offsetX + 5

                v.setSlideCommandX(value)
            })

            v.valueSetter_.on('__pointerdown', () => v.isAdjusting = false)
        })
    }
}

window.EntryScreenFixer = class EntryScreenFixer {
    static Marker = Symbol('EntryScreenFixer')
    
    constructor() {
        // this.Entry = document.querySelector('iframe')?.contentWindow?.Entry
        this.Entry = window.Entry
      
        if(!this.Entry) throw new Error('이 스크립트는 만들기 화면에서는 동작하지 않습니다.')

        this.canvas = this.Entry.stage.canvas.canvas
    }
  
    ratioX = 0
    ratioY = 0

    resX = 0
    resY = 0

    webGL = null

    fix() {
        if(this.Entry.options.useWebGL) {
            this.webGL = new EntryScreenFixerWebGL(this, this.Entry)
            this.webGL.fix()
        } else {
            const resolution = [Math.ceil(screen.width), Math.ceil(screen.width * 9 / 16)]
    
            this.info(`Found Device Resolution ${screen.width}x${screen.height}`)
    
            this.setScreenResolution(...resolution)
    
            this.fixSVG()
        }
        
        this.Entry.variableContainer.getVariableByName('.BESEnable')?.setValue?.('1')
    }

    fixSVG() {
        const scrFixer = this
        const { Entry } = this

        let entries = 0

        Entry.container.objects_.forEach(obj => {
            if(obj.esf_marker == EntryScreenFixer.Marker) return
            if(obj.objectType != 'sprite') return

            obj.esf_marker = EntryScreenFixer.Marker

            const svgImages = new Map()

            obj.pictures.forEach((pic, idx) => {
                if(pic.imageType == 'svg' && !pic.fileurl) {
                    const id = pic.filename
                    const image = new Image()

                    image.src = `./project/temp/${id.slice(0, 2)}/${id.slice(2, 4)}/image/${id}.svg`

                    svgImages.set(idx, [image, null])

                    fetch(image.src).then(resp => resp.text()).then(resp => (new DOMParser().parseFromString(resp, 'image/svg+xml'))).then(doc => {
                        const svg = [...doc.childNodes].find(x => x.tagName == 'svg')
                        const vbox = [svg.viewBox.baseVal.x, svg.viewBox.baseVal.y, svg.viewBox.baseVal.width, svg.viewBox.baseVal.height]
                        svgImages.set(idx, [svgImages.get(idx)[0], vbox[2] != 0 && vbox[3] != 0 ? vbox : [0, 0, svg.width.baseVal.value, svg.height.baseVal.value]])
                    })

                    entries++
                }
            })

            Object.defineProperty(obj.entity.object, 'draw', {
                get() {
                    return ctx => {
                        if(ctx.canvas != scrFixer.canvas) {
                            ctx.drawImage(img, 0, 0)
                            
                            return
                        }
                        
                        const w = obj.entity.getWidth()
                        const h = obj.entity.getHeight()

                        if((() => {
                            if(obj.entity.picture.imageType == 'svg') {
                                const image = svgImages.get(obj.pictures.indexOf(obj.entity.picture))

                                if(!image) throw new ReferenceError('[EntryScreenFixer] Unindexed SVG Item: ' + obj.getPictureIndex())

                                const viewBox = image[1]

                                if(!viewBox) return false

                                const [ _x, _y, vw, vh ] = viewBox

                                if(w != vw || h != vh) {
                                    const regX = vw / 2
                                    const regY = vh / 2
                                    const ssx = Entry.stage._app.stage.scaleX
                                    const ssy = Entry.stage._app.stage.scaleY
                                    const sx = obj.entity.scaleX * ssx
                                    const sy = obj.entity.scaleY * ssy
                                    const x = obj.entity.x
                                    const y = obj.entity.y
                                    const rot = obj.entity.rotation / 180 * Math.PI

                                    const tx = scrFixer.resX / 2 + x / 480 * scrFixer.resX
                                    const ty = scrFixer.resY / 2 - y / 270 * scrFixer.resY

                                    ctx.setTransform(
                                        sx, 0, 
                                        0, sy, 
                                        0, 0
                                    )

                                    ctx.translate(tx / sx, ty / sy)
                                    ctx.rotate(rot)
                                    ctx.translate(-regX, -regY)

                                    ctx.drawImage(image[0], 0, 0, vw, vh)

                                    return true
                                }
                            }
                        })()) return
                        
                        if(obj.entity.picture.imageType == 'svg') {
                            const image = svgImages.get(obj.pictures.indexOf(obj.entity.picture))

                            if(!image) throw new ReferenceError('[EntryScreenFixer] Unindexed SVG Item: ' + obj.getPictureIndex())

                            ctx.drawImage(image[0], 0, 0, w, h)
                        } else {
                            ctx.drawImage(obj.entity.object.image, 0, 0, w, h)
                        }
                    }
                }
            })

            let img = obj.entity.object.image
        })

        this.info(`Fixed ${entries} SVG Entries`)
    }

    setScreenResolution(w, h) {
        this.info(`Setting Rendering Resolution to ${w}x${h}`)

        const { Entry } = this
        const { stage } = Entry
    
        stage.canvas.canvas.width = w
        stage.canvas.canvas.height = h
        stage.canvas.x = w / 2
        stage.canvas.y = h / 2
        stage.canvas.scaleX = stage.canvas.scaleY = w / 240 / 2
        
        this.resX = w
        this.resY = h
    
        stage._app.stage.update()

        this.ratioX = 480 / w
        this.ratioY = 270 / h

        this.fixRatio()
    }

    fixRatio() {
        const { Entry } = this

        Entry.variableContainer.lists_.forEach(list => {
            list.view_.removeAllEventListeners()
            list.resizeHandle_.removeAllEventListeners()
            list.scrollButton_.removeAllEventListeners()

            list.view_.on('mouseover', () => list.view_.cursor = 'move')
            list.view_.on('mousedown', e => {
                if (Entry.type != 'workspace' || list.isResizing) return

                list.view_.offset = {
                    x: list.view_.x - (e.stageX * this.ratioX - 240),
                    y: list.view_.y - (e.stageY * this.ratioY - 135)
                }

                list.view_.cursor = 'move'
            })
            list.view_.on('pressup', () => {
                list.view_.cursor = 'initial'
                list.isResizing = false
            })
            list.view_.on('pressmove', e => {
                if (Entry.type != 'workspace' || list.isResizing) return

                list.setX(e.stageX * this.ratioX - 240 + list.view_.offset.x)
                list.setY(e.stageY * this.ratioY - 135 + list.view_.offset.y)
                list.updateView()
            })

            list.resizeHandle_.on('mouseover', () => list.resizeHandle_.cursor = 'nwse-resize')
            list.resizeHandle_.on('mousedown', e => {
                list.isResizing = true
            
                list.resizeHandle_.offset = {
                    x: e.stageX * this.ratioX - list.getWidth(),
                    y: e.stageY * this.ratioY - list.getHeight()
                }

                list.view_.cursor = 'nwse-resize'
            })
            list.resizeHandle_.on('pressmove', e => {
                list.setWidth(e.stageX * this.ratioX - list.resizeHandle_.offset.x)
                list.setHeight(e.stageY * this.ratioY - list.resizeHandle_.offset.y)
                list.updateView()
            })

            list.scrollButton_.on('mousedown', e => {
                list.isResizing = true
                list.scrollButton_.offsetY = e.stageY - list.scrollButton_.y / this.ratioY
            })
            list.scrollButton_.on('pressmove', e => {
                list.scrollButton_.y = Math.min(Math.max((e.stageY - list.scrollButton_.offsetY) * this.ratioY, 25), list.getHeight() - 30)
                list.updateView()
            })
        })

        Entry.variableContainer.variables_.forEach(v => {
            v.view_.removeAllEventListeners()

            v.view_.on('mousedown', e => {
                if (Entry.type != 'workspace') return

                v.view_.offset = {
                    x: v.view_.x - (e.stageX * this.ratioX - 240),
                    y: v.view_.y - (e.stageY * this.ratioY - 135)
                }
            })

            v.view_.on('pressmove', e => {
                if (Entry.type != 'workspace') return

                v.setX(e.stageX * this.ratioX - 240 + v.view_.offset.x)
                v.setY(e.stageY * this.ratioY - 135 + v.view_.offset.y)
                v.updateView()
            })

            if(!v.slideBar_) return

            v.slideBar_.removeAllEventListeners()
            v.valueSetter_.removeAllEventListeners()

            v.slideBar_.on('mousedown', e => {
                if (!Entry.engine.isState('run')) return

                const value = evt.stageX * this.ratioX - (v.slideBar_.getX() + 240 + 5) + 5

                v.setSlideCommandX(value)
            })

            v.valueSetter_.on('mousedown', e => {
                if (!Entry.engine.isState('run')) return

                v.isAdjusting = true
                v.valueSetter_.offsetX = e.stageX * this.ratioX - v.valueSetter_.x
            })
    
            v.valueSetter_.on('pressmove', e => {
                if (!Entry.engine.isState('run')) return

                const value = (e.stageX * this.ratioX) - v.valueSetter_.offsetX + 5

                v.setSlideCommandX(value)
            })

            v.valueSetter_.on('pressup', () => v.isAdjusting = false)
        })
    }

    info(message) {
        console.log('%c EntryScreenFixer %c INFO %c ' + message, 'background: black; color: white; border-radius: 5px 0px 0px 5px;', 'background: #08c490; color: white; border-radius: 0px 5px 5px 0px;', '')
    }
}

// new EntryScreenFixer().fix()
