import React, { useRef, useEffect, createContext, useContext, useState, FC } from 'react'

export const CanvasContext = createContext<{
    context: CanvasRenderingContext2D | undefined
}>({
    context: undefined,
})

export const useCanvasContext = () => {
    return useContext(CanvasContext)
}

export const Canvas: FC = (props) => {

    const [width, setWidth] = useState(0);
    const [height, setHeight] = useState(0);
    const pixelRatio = window.devicePixelRatio;
    const ref = useRef<HTMLDivElement>(null);

    const canvasRef = useRef<HTMLCanvasElement>(null)
    const [context, setContext] = useState<CanvasRenderingContext2D | undefined>()


    // useEffect(() => {
    //     const ctx = canvasRef?.current?.getContext('2d')
    //     if (ctx) setContext(ctx)
    // }, [])

    const draw = (ctx: CanvasRenderingContext2D, frameCount: number) => {
        ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height)
        ctx.fillStyle = '#fff'
        ctx.beginPath()
        ctx.arc(50, 100, 20 * Math.sin(frameCount * 0.05) ** 2, 0, 2 * Math.PI)
        ctx.fill()
    }

    useEffect(() => {
        const ctx = canvasRef?.current?.getContext('2d')
        if (ctx) setContext(ctx)

        setWidth(ref?.current?.clientWidth);
        setHeight(ref?.current?.clientHeight > 400 ? ref?.current?.clientHeight : 400);

        let frameCount = 0
        let animationFrameId: any

        const render = () => {
            frameCount++
            if (ctx) draw(ctx, frameCount)
            animationFrameId = window.requestAnimationFrame(render)
        }
        render()

        return () => { window.cancelAnimationFrame(animationFrameId) }
    }, [draw])


    const displayWidth = Math.floor(pixelRatio * width);
    const displayHeight = Math.floor(pixelRatio * height);
    const style = { width, height };

    return <div style={{ width: '100%' }} ref={ref}>
        <canvas
            ref={canvasRef}
            // width={displayWidth}
            // height={displayHeight}
            // style={style}

            {...props} />
    </div>
}
